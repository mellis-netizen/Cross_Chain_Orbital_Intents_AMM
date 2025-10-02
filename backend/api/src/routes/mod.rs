pub mod intents;
pub mod solver;
pub mod analytics;
pub mod health;
pub mod auth;

use axum::Router;
use crate::models::AppState;

// Create the main API router
pub fn create_api_router() -> Router<AppState> {
    Router::new()
        .nest("/api/v1/intents", intents::routes())
        .nest("/api/v1/solver", solver::routes())
        .nest("/api/v1/analytics", analytics::routes())
        .nest("/api/v1/auth", auth::routes())
        .merge(health::routes())
}