use std::sync::LazyLock;
use crate::config::CONFIG;
use super::Session;
use deadpool_redis::{
    redis::cmd,
    Config,
    Runtime,
    Pool
};

use tracing::error;

#[cfg(feature = "redis")]
pub static POOL: LazyLock<Pool> = LazyLock::new(|| {
    let cfg = Config::from_url(&CONFIG.session.redis_url);
    cfg.create_pool(Some(Runtime::Tokio1)).unwrap()
});

#[cfg(feature = "redis")]
pub struct RedisSession {
    pub pool: &'static LazyLock<Pool>,
}

#[cfg(feature = "redis")]
impl RedisSession {
    #[must_use]
    pub fn new() -> Self {
        Self { pool: &POOL }
    }
}

#[cfg(feature = "redis")]
impl Session for RedisSession {
    #[allow(clippy::must_use_candidate)]
    async fn contains(&self, circuit_id: u32) -> bool {
        let mut conn = match self.pool.get().await {
            Ok(conn) => conn,
            Err(e) => {
                error!(error = ?e, "Failed to get connection from pool, blocking access for safety.");
                return false
            },
        };

        match cmd("EXISTS").arg(circuit_id).query_async::<i64>(&mut conn).await {
            Ok(v) => v > 0,
            Err(e) => {
                error!(error = ?e, "Redis error, blocking access for safety.");
                false
            }
        }
    }

    async fn set(&self, circuit_id: u32) {
        let mut conn = match self.pool.get().await {
            Ok(conn) => conn,
            Err(e) => {
                error!(error = ?e, "Failed to get connection from pool.");
                return;
            },
        };

        match cmd("SET")
            .arg(circuit_id)
            .arg("")
            .arg("EX")
            .arg(CONFIG.session.ttl)
            .query_async::<()>(&mut conn)
            .await {
            Ok(()) => {}
            Err(e) => {
                error!(error = ?e, "Redis error.");
            }
        }

    }
}

impl Default for RedisSession {
    fn default() -> Self {
        Self::new()
    }
}