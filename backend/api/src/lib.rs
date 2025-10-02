pub mod routes;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod database;
pub mod cache;
pub mod websocket;
pub mod metrics;
pub mod auth;
pub mod error;
pub mod config;

pub use config::Config;
pub use error::{ApiError, Result};

use axum::{Router, middleware as axum_middleware};
use tower::ServiceBuilder;
use tower_http::{
    cors::{CorsLayer, Any},
    trace::TraceLayer,
    compression::CompressionLayer,
    timeout::TimeoutLayer,
};
use std::time::Duration;
use metrics_exporter_prometheus::PrometheusBuilder;

pub async fn create_app(config: Config) -> Result<Router> {
    // Initialize metrics
    let prometheus_handle = PrometheusBuilder::new()
        .install()
        .expect("failed to install Prometheus recorder");

    // Initialize database
    let db_pool = database::create_pool(&config.database_url).await?;
    database::run_migrations(&db_pool).await?;

    // Initialize Redis
    let redis_client = cache::create_client(&config.redis_url).await?;

    // Initialize intents engine
    let intents_engine = intents_engine::IntentsEngine::new(config.chains.clone()).await
        .map_err(|e| ApiError::Internal(format!("Failed to initialize intents engine: {}", e)))?;

    // Create application state
    let app_state = models::AppState {
        db: db_pool,
        redis: redis_client,
        intents_engine,
        config: config.clone(),
        prometheus_handle,
    };

    // Build middleware stack
    let middleware = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any))
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            middleware::rate_limit
        ))
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            middleware::auth
        ));

    // Build router
    let app = Router::new()
        .merge(routes::intents::routes())
        .merge(routes::solver::routes())
        .merge(routes::analytics::routes())
        .merge(routes::health::routes())
        .route("/ws", axum::routing::get(websocket::websocket_handler))
        .layer(middleware)
        .with_state(app_state);

    Ok(app)
}

pub async fn start_server(config: Config) -> Result<()> {
    let app = create_app(config.clone()).await?;
    
    let listener = tokio::net::TcpListener::bind(&config.server_address)
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to bind to address: {}", e)))?;

    tracing::info!("Server starting on {}", config.server_address);
    
    axum::serve(listener, app)
        .await
        .map_err(|e| ApiError::Internal(format!("Server error: {}", e)))?;

    Ok(())
}