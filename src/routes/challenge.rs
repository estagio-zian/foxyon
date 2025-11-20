use super::get_circuit_id;

use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    pow::{
        Challenge,
        CHALLENGE_LEN,
        B64_LEN,
        MIN_SOLUTION_LEN,
        check_integrity,
        validate_challenge
    },
    session::{
        SessionCache,
        Session,
        challenge_blacklist::ChallengeBlacklist
    }
};

use actix_web::{
    HttpResponse,
    HttpRequest,
    Result,
    error,
    web
};
use actix_web::error::ErrorInternalServerError;
use actix_web::http::header;
use actix_web::web::Bytes;
use base64_simd::{STANDARD_NO_PAD, Out};
use sailfish::TemplateOnce;
use serde::Deserialize;
use tracing::error;
use tracing::debug;
use tokio::sync::watch::Receiver;
use memchr::{memchr_iter, memchr};
use ada_url::Url;

const MAX_SOLUTION_LENGTH: usize = 150;


#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Solution {
    solution: String,
}

const DUMMY_URL: &str = "http://a.b/?";

/// # Errors
/// Will return `Err` if there is an error rendering the challenge template,
/// returning an `InternalServerError` response
#[allow(clippy::unused_async)]
pub async fn challenge_page(cpu_usage: web::Data<Receiver<f32>>) -> Result<HttpResponse> {
    let body = Challenge::new(cpu_usage.as_ref()).render_once().map_err(|e| {
        error!(error = ?e, "Failed to render challenge template");
        ErrorInternalServerError("Failed to render challenge template")
    })?;
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(body))
}

/// # Errors
/// Will return `Err` if validation fails or does not receive the appropriate header, such as ‘X-circuit-id’.
/// returning an `ErrorBadRequest` or `ErrorUnauthorized` response
pub async fn challenge_post(
    form: web::Bytes,
    req: HttpRequest,
    session: web::Data<SessionCache>,
    blacklist: web::Data<ChallengeBlacklist>) -> Result<HttpResponse, error::Error>
{

    let circuit_id = get_circuit_id(req.headers().get("X-Circuit-Id"))?;

    let (nonce, challenge, difficulty_bits, expires_at, integrity_base64) = validate_and_get_user_input(&form)?;

    let challenge_bytes: [u8; CHALLENGE_LEN] = match challenge.try_into() {
        Ok(bytes) => bytes,
        Err(_) => {
            return Err(SolutionError::MalformedInput("Challenge contains more bytes than originally sent").into());
        }
    };

    // TODO
    if !blacklist.try_insert(challenge_bytes).await {
        return Err(SolutionError::Blacklisted.into());
    }

    let mut client_integrity_buf = [0u8; 32];
    let client_integrity = STANDARD_NO_PAD.decode(integrity_base64.as_ref(), Out::from_slice(&mut client_integrity_buf))
    .map_err(|_| SolutionError::MalformedInput("Base64 validation failed"))?;

    if !check_integrity(challenge, difficulty_bits, expires_at, client_integrity) {
        return Err(SolutionError::MalformedInput("Integrity check failed").into());
    }

    let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_secs(),
        Err(e) => {
            error!(error = ?e, "System time is before UNIX_EPOCH");
            return Err(SolutionError::InternalError.into());
        }
    };

    if expires_at < now {
        return Err(SolutionError::TimedOut.into());
    }

    if validate_challenge(nonce, challenge, difficulty_bits, expires_at) {
        session.set(circuit_id).await;


        let original_uri = req.headers()
            .get("X-Original-URI")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("/");

        #[cfg(feature = "debug")]
        debug!("Original URI: {}", original_uri);

        Ok(HttpResponse::SeeOther().insert_header((header::LOCATION, original_uri)).finish())
    } else {
        Err(SolutionError::ValidationFailed.into())
    }
}

// #[inline]
// #[allow(clippy::arithmetic_side_effects, clippy::type_complexity)]
// fn validate_and_get_user_input(solution: &Bytes) -> Result<(&[u8], &[u8], u8, u64, &[u8]), SolutionError> {
//     const OFFSET: usize = 9;
//
//     if solution.len() < OFFSET {
//         return Err(SolutionError::MalformedInput("Solution too short"));
//     }
//
//     let mut pipes = memchr_iter(b'%', &solution[OFFSET..]);
//
//     // índices absolutos
//     let p1 = pipes.next().ok_or(SolutionError::MalformedInput("Unable to obtain the nonce"))? + OFFSET;
//     let p2 = pipes.next().ok_or(SolutionError::MalformedInput("Unable to retrieve the challenge"))? + OFFSET;
//     let p3 = pipes.next().ok_or(SolutionError::MalformedInput("Unable to obtain the difficulty"))? + OFFSET;
//     let p4 = pipes.next().ok_or(SolutionError::MalformedInput("Data integrity could not be obtained"))? + OFFSET;
//
//     // valida limites antes do slicing
//     if p1+3 > p2 || p2+3 > p3 || p3+3 > p4 || p4+3 > solution.len() {
//         return Err(SolutionError::MalformedInput("Malformed slices"));
//     }
//
//     let nonce = &solution[OFFSET..p1];
//     let challenge = &solution[p1+3..p2];
//     let difficulty_str = &solution[p2+3..p3];
//     let expires_str = &solution[p3+3..p4];
//     let integrity_b64 = &solution[p4+3..];
//
//     if nonce.is_empty() || challenge.len() != CHALLENGE_LEN {
//         return Err(SolutionError::MalformedInput("Validation failed"));
//     }
//
//     let difficulty_bits = atoi_simd::parse::<u8>(difficulty_str)
//         .map_err(|_| SolutionError::MalformedInput("atoi difficulty failed"))?;
//     let expires_at = atoi_simd::parse::<u64>(expires_str)
//         .map_err(|_| SolutionError::MalformedInput("atoi expires_at failed"))?;
//
//     Ok((nonce, challenge, difficulty_bits, expires_at, integrity_b64))
// }

#[inline]
#[allow(clippy::arithmetic_side_effects, clippy::type_complexity)]
fn validate_and_get_user_input(solution: &Bytes) -> Result<(&[u8], &[u8], u8, u64, &[u8]), SolutionError> {
    if  solution.len() <= MIN_SOLUTION_LEN
        || memchr::memchr(b'\x80', solution).is_some()
        || !solution.starts_with(b"solution=") {
        return Err(SolutionError::MalformedInput("Field validation failure"));
    }

    // SAFETY: Validation is performed at the beginning of the function; we do not need to validate again.
    let query_str = unsafe {std::str::from_utf8_unchecked(solution)};

    let url = Url::parse(DUMMY_URL, Some(query_str)).map_err(|_| SolutionError::MalformedInput(""))?;



}

#[derive(Debug, Clone, PartialEq)]
enum SolutionError {
    MalformedInput(&'static str),
    ValidationFailed,
    Blacklisted,
    CircuitIdError,
    InternalError,
    TimedOut,
}

impl From<SolutionError> for actix_web::Error {
    fn from(err: SolutionError) -> actix_web::Error {
        match err {
            SolutionError::MalformedInput(msg) => {
                error::ErrorForbidden(msg)
            }
            SolutionError::ValidationFailed => {
                error::ErrorBadRequest("Solution validation failed")
            }
            SolutionError::Blacklisted => {
                error::ErrorForbidden("Blacklisted challenge")
            }
            SolutionError::CircuitIdError => {
                error::ErrorForbidden("CircuitID already in use")
            }
            SolutionError::InternalError => {
                error::ErrorInternalServerError("Internal error")
            }
            SolutionError::TimedOut => {
                error::ErrorForbidden("The challenge has expired!")
            }
        }
    }
}