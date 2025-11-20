pub mod challenge;
mod kill;
pub mod auth;


use std::net::Ipv6Addr;
use actix_web::{error, Result};
use actix_web::http::header::HeaderValue;
use tracing::error;

/// Extracts a Tor circuit ID (`u32`) from the `X-Circuit-Id` header.
///
/// The Tor circuit ID is provided as an IPv6-style string (e.g. `fc00:dead:beef:4dad::12d`).
/// This function parses it as an IPv6 address and derives the numeric circuit ID
/// from the last 4 bytes of the address.
///
/// # Errors
///
/// Will return `ErrorInternalServerError` if validation of `X-Circuit-Id` fails.
pub fn get_circuit_id(circuit_id_header: Option<&HeaderValue>) -> Result<u32, actix_web::Error> {
    let circuit_id_str = if let Some(h) = circuit_id_header {
        h
            .to_str()
            .map_err(|e| {
                error!(error = ?e, "Error parsing circuit_id");
                error::ErrorInternalServerError("Invalid X-Circuit-Id header")
            })?
    } else {
        error!("X-Circuit-Id header is missing");
        return Err(error::ErrorInternalServerError("X-Circuit-Id header is missing"))
    };

    let ipv6: Ipv6Addr = circuit_id_str
        .parse()
        .map_err(|_| error::ErrorInternalServerError("Invalid X-Circuit-Id header"))?;
    let octs: [u8; 16] = ipv6.octets();
    let circuit_id = u32::from_be_bytes([octs[12], octs[13], octs[14], octs[15]]);
    Ok(circuit_id)
}