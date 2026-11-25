mod alerts;
mod auth;
mod devices;
mod health;
mod telemetry;

use axum::{middleware, routing::{get, post}, Router};
use tower_http::{limit::RequestBodyLimitLayer, trace::TraceLayer};
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    let protected = Router::new()
        .route("/devices", post(devices::register).get(devices::list))
        .route("/telemetry", post(telemetry::ingest).get(telemetry::query))
        .route("/alert-rules", post(alerts::create).get(alerts::list))
        .route_layer(middleware::from_fn_with_state(state.clone(), crate::auth::require_auth));
    Router::new()
        .route("/health", get(health::health))
        .route("/auth/login", post(auth::login))
        .merge(protected)
        .layer(RequestBodyLimitLayer::new(2 * 1024 * 1024))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
