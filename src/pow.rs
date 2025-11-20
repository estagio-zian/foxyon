use std::time::{SystemTime, UNIX_EPOCH};
use crate::crypto::blake3::{pow_integrity_hash, pow_challenge_hash};
use crate::config::CONFIG;

use sailfish::TemplateOnce;
use rand::{Rng, distr::Alphanumeric};
use base64_simd::{STANDARD_NO_PAD, Out};
use tracing::{error, info};
use subtle::ConstantTimeEq;
use primitive_types::U256;
use tokio::sync::watch::Receiver;

pub const CHALLENGE_LEN: usize = 12;
pub const B64_LEN: usize = 43;
pub const TIMESTAMP_LEN: usize = 10;
pub const MIN_SOLUTION_LEN: usize = 82;

#[derive(TemplateOnce)]
// TODO
#[cfg_attr(not(feature = "debug"), template(path = "challenge.html"))]
#[cfg_attr(feature = "debug", template(path = "challenge.html"))]
pub struct Challenge {
    pub challenge: [u8; CHALLENGE_LEN],
    pub difficulty_bits: u8,
    pub expires_at: u64,
    pub integrity_b64: [u8; B64_LEN],
}

impl Challenge {
    pub fn new(cpu_usage: &Receiver<f32>) -> Challenge {
        let challenge: [u8; CHALLENGE_LEN] = {
            let mut rng = rand::rng();
            std::array::from_fn(|_| rng.sample(Alphanumeric))
        };
        let difficulty_bits: u8 = match *cpu_usage.borrow() {
            cpu if cpu < CONFIG.pow.cpu_thresholds.low => CONFIG.pow.difficulty.minimum,
            cpu if cpu <  CONFIG.pow.cpu_thresholds.medium => CONFIG.pow.difficulty.medium,
            cpu if cpu < CONFIG.pow.cpu_thresholds.high => CONFIG.pow.difficulty.high,
            _ => CONFIG.pow.difficulty.ultra
        };

        #[cfg(feature = "debug")]
        info!("Challenge difficulty bits is {difficulty_bits} and CPU usage at {}", *cpu_usage.borrow());

        let expires_at: u64 = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(d) => d.as_secs(),
            Err(e) => {
                error!(error = ?e, "System time is before UNIX_EPOCH; using 0 as fallback for expiration");
                0
            },
        }.saturating_add(CONFIG.pow.challenge_ttl);


        let integrity_b64: [u8; B64_LEN] = {
            let mut buf = [0u8; B64_LEN];
            let _ = STANDARD_NO_PAD.encode(&pow_integrity_hash(&challenge, difficulty_bits, expires_at), Out::from_slice(&mut buf));
            buf
        };

        Challenge {
            challenge,
            difficulty_bits,
            expires_at,
            integrity_b64,
        }
    }

    #[inline]
    #[must_use]
    pub fn challenge_str(&self) -> &str {
        debug_assert!(std::str::from_utf8(&self.challenge).is_ok());
        // SAFETY: `Alphanumeric` contains only ASCII characters
        unsafe { std::str::from_utf8_unchecked(&self.challenge) }
    }

    #[inline]
    #[must_use]
    pub fn integrity_b64_str(&self) -> &str {
        debug_assert!(std::str::from_utf8(&self.integrity_b64).is_ok());
        // SAFETY: `Base64` contains only ASCII characters
        unsafe { std::str::from_utf8_unchecked(&self.integrity_b64) }
    }
}

#[inline]
#[must_use]
pub fn check_integrity(challenge: &[u8], difficulty_bits: u8, expires_at: u64, client_integrity: &[u8]) -> bool {
    pow_integrity_hash(challenge, difficulty_bits, expires_at)
        .ct_eq(client_integrity).into()
}

#[inline]
#[must_use]
pub fn validate_challenge(client_work: &[u8], challenge: &[u8], difficulty_bits: u8, expires_at: u64) -> bool {
    U256::from_little_endian(&pow_challenge_hash(client_work, challenge, expires_at)).trailing_zeros() >= difficulty_bits.into()
}