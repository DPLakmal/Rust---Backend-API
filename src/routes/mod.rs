use axum::{
    middleware,
    routing::get,
    Router,
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::{
    handlers::{health::health_check, items::{create_item, delete_item, get_item, list_items, update_item}},
    middleware::auth::auth_middleware,
    state::AppState,
};

pub fn create_router(state: AppState) -> Router {
    // Protected API routes guarded by Gateway Auth Middleware
    let protected_routes = Router::new()
        .route("/items", get(list_items).post(create_item))
        .route("/items/:id", get(get_item).put(update_item).delete(delete_item))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Public routes (Health check & welcome endpoint)
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/", get(|| async { "Rust Backend REST API is running!" }));

    // CORS layer (Gateway configuration)
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .merge(public_routes)
        .nest("/api", protected_routes)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
