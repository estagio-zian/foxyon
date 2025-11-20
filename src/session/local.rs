#[cfg(feature = "local")]
use std::{
    hash::BuildHasherDefault,
    time::Duration
};

#[cfg(feature = "local")]
use crate::{
    config::CONFIG,
    session::Session
};

#[cfg(feature = "local")]
use moka::future::Cache;
#[cfg(feature = "local")]
use twox_hash::XxHash3_64;

#[cfg(feature = "local")]
pub struct MokaSession {
    pub cache: Cache<u32, (), BuildHasherDefault<XxHash3_64>>,
}

#[cfg(feature = "local")]
impl MokaSession {
    #[must_use]
    pub fn new() -> Self {
        Self { cache:
        Cache::builder()
            .initial_capacity(CONFIG.session.initial_capacity)
            .max_capacity(CONFIG.session.max_capacity)
            .time_to_live(Duration::from_secs(CONFIG.session.ttl))
            .time_to_idle(Duration::from_secs(CONFIG.session.tti))
            .build_with_hasher(BuildHasherDefault::<XxHash3_64>::default())
        }
    }
}

#[cfg(feature = "local")]
impl Session for MokaSession {
    async fn contains(&self, circuit_id: u32) -> bool {
        self.cache.contains_key(&circuit_id)
    }

    async fn set(&self, circuit_id: u32) {
        self.cache.insert(circuit_id, ()).await;
    }
}

#[cfg(feature = "local")]
impl Default for MokaSession {
    fn default() -> Self {
        Self::new()
    }
}