use fleet_telemetry_api::{build_router, config::Config, AppState};
use axum::{body::Body, http::{Request, StatusCode}};
use sqlx::postgres::PgPoolOptions;
use std::collections::HashMap;
use tower::ServiceExt;

fn test_config() -> Config {
    let values: HashMap<String,String> = [
        ("DATABASE_URL","postgres://unused"),("JWT_SECRET","0123456789abcdef0123456789abcdef")
    ].into_iter().map(|(a,b)|(a.into(),b.into())).collect();
    Config::from_lookup(|k|values.get(k).cloned()).unwrap()
}
#[tokio::test]
async fn health_route_is_registered() {
    // This smoke test intentionally uses a lazy pool; health correctly reports
    // a database error if no local PostgreSQL server is available.
    let pool=PgPoolOptions::new().connect_lazy("postgres://postgres:postgres@localhost/fleet_telemetry").unwrap();
    let app=build_router(AppState::new(pool,test_config()));
    let response=app.oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap()).await.unwrap();
    assert!(response.status()==StatusCode::OK || response.status()==StatusCode::INTERNAL_SERVER_ERROR);
}
#[tokio::test]
async fn protected_endpoint_rejects_missing_token() {
    let pool=PgPoolOptions::new().connect_lazy("postgres://postgres:postgres@localhost/fleet_telemetry").unwrap();
    let app=build_router(AppState::new(pool,test_config()));
    let response=app.oneshot(Request::builder().method("GET").uri("/devices").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(response.status(),StatusCode::UNAUTHORIZED);
}
