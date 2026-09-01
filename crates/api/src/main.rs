use std::net::SocketAddr;
use infrastructure::config::AppConfig;
use api::{server::{build_router, shutdown_signal}, state::AppState};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,api=debug,tower_http=debug".into()))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    tracing::info!("Loading platform configuration...");
    let config = AppConfig::load()?;

    tracing::info!("Running database migrations...");
    let db = infrastructure::database::PostgresDatabase::new(&config).await?;
    db.run_migrations().await?;

    tracing::info!("Initializing application state...");
    let state = AppState::init(config.clone()).await?;

    let app = build_router(state);

    let addr: SocketAddr = format!("{}:{}", config.server_host, config.server_port).parse()?;
    tracing::info!("Platform HTTP server running on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Server shut down gracefully.");
    Ok(())
}