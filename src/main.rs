use std::str::FromStr;
use std::time::Duration;
use foxyon::{
    config::CONFIG,
    routes::{
        auth::auth,
        challenge::{challenge_page, challenge_post}
    },
    session::{
        SessionCache,
        challenge_blacklist::ChallengeBlacklist
    },
    system::cpu_usage,
};

use tokio::{sync::watch, task};

use actix_web::{web, App, HttpServer};
use tracing::Level;
use tracing_subscriber::fmt;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    fmt().with_max_level(Level::from_str(&CONFIG.logging.level).unwrap_or_else(|_|
        {
            eprintln!("Invalid log level '{}' in configuration; falling back to ERROR.", &CONFIG.logging.level);
            Level::ERROR
        }))
        .init();

    let (tx_cpu_usage, rx_cpu_usage) = watch::channel(0f32);

    task::spawn(async move {
        cpu_usage(tx_cpu_usage).await;
    });

    let session = web::Data::new(SessionCache::new());
    let nonce_filter = web::Data::new(ChallengeBlacklist::default());
    let cpu_usage = web::Data::new(rx_cpu_usage);

    HttpServer::new(move || {
        App::new()
            .app_data(cpu_usage.clone())
            .app_data(session.clone())
            .app_data(nonce_filter.clone())
            .route(&CONFIG.routes.challenge, web::get().to(challenge_page))
            .route(&CONFIG.routes.auth, web::get().to(auth))
            .route(&CONFIG.routes.challenge, web::post().to(challenge_post))
    })
        .bind(format!("{}:{}", &CONFIG.server.host, &CONFIG.server.port))?
        .backlog(CONFIG.server.backlog)
        .max_connections(CONFIG.server.max_connections)
        .keep_alive(Duration::from_secs(CONFIG.server.keep_alive))
        .workers(CONFIG.server.workers).run().await
}