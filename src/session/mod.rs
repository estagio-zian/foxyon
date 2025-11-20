pub mod challenge_blacklist;


#[cfg(feature = "local")]
mod local;
#[cfg(feature = "local")]
pub use local::MokaSession as SessionCache;

#[cfg(feature = "redis")]
mod redis;

#[cfg(feature = "redis")]
pub use redis::RedisSession as SessionCache;

use std::future::Future;

pub trait Session {
    fn contains(&self, circuit_id: u32) -> impl Future<Output = bool>;
    fn set(&self, circuit_id: u32) -> impl Future<Output = ()>;
}
