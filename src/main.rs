use std::sync::Arc;

use sqlx::PgPool;
use tokio::signal;

pub(crate) mod config;
pub(crate) mod db;
pub(crate) mod error;
pub(crate) mod identifier;
pub(crate) mod net;

use config::Config;
use net::{model, mojang::MojangAuth, rate_limit::RateLimiter};

#[derive(Clone)]
struct AppState {
    pub config: Config,
    pub db_pool: PgPool,
    pub mojang: MojangAuth,
    pub auth_limiter: Arc<RateLimiter>,
    pub data_limiter: Arc<RateLimiter>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok(); //load .env if present

    let config = Config::from_env();

    let db_pool = db::init_pool(&config.db_url)
        .await
        .map_err(|e| {
            eprintln!("ERROR: database not working");
            e
        })?;

    db::run_migrations(&db_pool)
        .await
        .map_err(|e| {
            eprintln!("ERROR: migrations failed");
            e
        })?;

    let state = Arc::new(AppState {
        config,
        db_pool,
        mojang: MojangAuth::new(),
        auth_limiter: Arc::new(RateLimiter::new(10, 60)),
        data_limiter: Arc::new(RateLimiter::new(60, 60)),
    });

    let port = state.config.port;

    let app =
        net::setup_routes(state).into_make_service_with_connect_info::<std::net::SocketAddr>();

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .map_err(|e| {
            eprintln!("ERROR: failed to bind port");
            e
        })?;

    println!("INFO: Persista starting on port {port}");

    #[cfg(target_family = "unix")]
    let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())
        .map_err(|e| {
            eprintln!("ERROR: failed to install SIGTERM handler");
            e
        })?;
    #[cfg(target_family = "windows")]
    let mut sigterm = signal::windows::ctrl_close()
        .map_err(|e| {
            eprintln!("ERROR: failed to install SIGTERM handler");
            e
        })?;

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            tokio::select! {
                _ = sigterm.recv() => {
                    println!("INFO: Received SIGTERM");
                    std::process::exit(0);
                },
                _ = signal::ctrl_c() => {
                    println!("INFO: Interrupted");
                    std::process::exit(130);
                },
            };
        })
        .await
        .map_err(|e| {
            eprintln!("ERROR: server error");
            e
        })?;
    
        Ok(())
}
