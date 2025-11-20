use std::hash::BuildHasherDefault;
use std::time::Duration;
use crate::config::CONFIG;
use moka::future::Cache;
use ahash::AHasher;

pub struct ChallengeBlacklist {
    inner: Cache<[u8; 12], (), BuildHasherDefault<AHasher>>,
}

impl ChallengeBlacklist {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner:
            Cache::builder()
                .initial_capacity(CONFIG.session.initial_capacity)
                .max_capacity(CONFIG.session.max_capacity)
                .time_to_live(Duration::from_secs(CONFIG.session.ttl))
                .build_with_hasher(BuildHasherDefault::<AHasher>::default())
        }
    }

    #[must_use]
    pub async fn try_insert(&self, challenge: [u8; 12]) -> bool {
        self.inner.entry(challenge)
            .or_insert(()).await
            .is_fresh()
    }




}

impl Default for ChallengeBlacklist {
    fn default() -> Self {
        Self::new()
    }
}