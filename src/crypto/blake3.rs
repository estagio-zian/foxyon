use std::sync::LazyLock;
use crate::config::CONFIG;
use rand::{TryRngCore, rngs::OsRng};
use tracing::{warn, info};

static BLAKE_KEY: LazyLock<[u8; 32]> = LazyLock::new(|| {
    let mut key: [u8; 32] = [0u8; 32];
    let external_password = &CONFIG.security.keyed_hash;
    if external_password.len() >= 32 {
        key.copy_from_slice(&external_password.as_bytes()[..32]);
    }
    else {
        if !external_password.is_empty() {
            warn!("Configured keyed_hash is too short (<32 bytes), falling back to securely generated random key");
        }
        OsRng.try_fill_bytes(&mut key).unwrap_or_else(|e| panic!("Secure key generation could not be guaranteed: {e}"));
        #[cfg(feature = "debug")]
        info!("The secure key has been generated.");
    }
    key
});

#[must_use]
pub fn pow_integrity_hash(challenge: &[u8], difficulty: u8, timestamp: u64) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new_keyed(&BLAKE_KEY);
    hasher.update(challenge);
    hasher.update(&[difficulty]);
    hasher.update(&timestamp.to_le_bytes());
    hasher.finalize().into()
}

#[must_use]
pub fn pow_challenge_hash(nonce: &[u8], challenge: &[u8], timestamp: u64) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(nonce);
    hasher.update(challenge);
    hasher.update(itoa::Buffer::new().format(timestamp).as_bytes());
    hasher.finalize().into()
}

mod tests {
    #[allow(unused_imports)]
    use super::*;
    #[test]
    fn pow_hash_is_deterministic(){
        let challenge: [u8;4] = *b"test";
        let difficulty: u8 = 69;
        let timestamp: u64 = 17_57_30_33_29;

        let hash1 = pow_integrity_hash(&challenge, difficulty, timestamp);
        let hash2 = pow_integrity_hash(&challenge, difficulty, timestamp);

        assert_eq!(hash1, hash2, "Equal inputs should generate equal outputs");
    }

    #[test]
    fn pow_hash_is_input_sensitive(){
        let challenge: [u8;4] = *b"test";
        let difficulty: u8 = 69;
        let timestamp: u64 = 17_57_30_33_29;
        let base = pow_integrity_hash(&challenge, difficulty, timestamp);

        let diff = pow_integrity_hash(b"123", difficulty, timestamp);
        assert_ne!(base, diff, "A different challenge should generate a different output");
        let diff = pow_integrity_hash(&challenge, 68, timestamp);
        assert_ne!(base, diff, "A different difficulty should generate a different output");
        let diff = pow_integrity_hash(&challenge, difficulty, 14_57_30_33_29);
        assert_ne!(base, diff, "A different timestamp should generate a different output.");
    }
}