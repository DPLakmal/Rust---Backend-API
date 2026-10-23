use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use rust_backend_api::{
    config::Config,
    db::init_db_pool,
    routes::create_router,
    state::AppState,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file if available
    dotenvy::dotenv().ok();

    // Initialize structured logging and tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rust_backend_api=debug,tower_http=debug,axum=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load application configuration
    let config = Config::from_env();
    tracing::info!(
        "Starting Rust Backend API with configuration: host={}, port={}",
        config.host,
        config.port
    );

    // Initialize Database Pool & run migrations
    let pool = init_db_pool(&config.database_url).await?;

    // Create shared application state
    let state = AppState::new(pool, config.clone());

    // Build the Axum router with Gateway middleware
    let app = create_router(state);

    // Bind server listener
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("Server listening on http://{}", addr);

    // Run server with graceful shutdown on Ctrl+C or SIGTERM
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Server shut down gracefully.");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C signal. Initiating graceful shutdown...");
        },
        _ = terminate => {
            tracing::info!("Received terminate signal. Initiating graceful shutdown...");
        },
    }
}
