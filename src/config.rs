use std::sync::LazyLock;

use serde::Deserialize;
use toml;

pub static CONFIG_FILE: &str = include_str!("../config.toml");

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    toml::from_str(CONFIG_FILE).unwrap_or_else(|e| { panic!("Error reading configuration file: {e}") })
});
#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: Server,
    pub routes: Routes,
    pub logging: Logging,
    pub pow: Pow,
    pub session: Session,
    pub security: Security,
    pub system: System,
}

#[derive(Debug, Deserialize)]
pub struct Server {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub backlog: u32,
    pub max_connections: usize,
    pub keep_alive: u64,
}

#[derive(Debug, Deserialize)]
pub struct Pow {
    pub challenge_ttl: u64,
    pub difficulty: Difficulty,
    pub cpu_thresholds: CpuThresholds
}

#[derive(Debug, Deserialize)]
pub struct Difficulty {
    pub minimum: u8,
    pub medium: u8,
    pub high: u8,
    pub ultra: u8
}

#[derive(Debug, Deserialize)]
pub struct CpuThresholds {
    pub low: f32,
    pub medium: f32,
    pub high: f32,
    pub critical: f32,
}

#[derive(Debug, Deserialize)]
pub struct Routes {
    pub auth: String,
    pub challenge: String,
}

#[derive(Debug, Deserialize)]
pub struct Logging {
    pub level: String,
}

#[derive(Debug, Deserialize)]
pub struct Session {
    pub redis_url: String,
    pub initial_capacity: usize,
    pub max_capacity: u64,
    pub tti: u64,
    pub ttl: u64,
}

#[derive(Debug, Deserialize)]
pub struct Security {
    pub keyed_hash: String,
}

#[derive(Debug, Deserialize)]
pub struct System {
    pub cpu_usage_update_interval: u64,
}