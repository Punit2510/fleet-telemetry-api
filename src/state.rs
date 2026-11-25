use std::sync::Arc;
use sqlx::PgPool;
use crate::{auth::JwtKeys, config::Config};
#[derive(Clone)]
pub struct AppState { pub pool: PgPool, pub config: Arc<Config>, pub jwt: Arc<JwtKeys> }
impl AppState {
    pub fn new(pool: PgPool, config: Config) -> Self {
        let jwt = JwtKeys::new(&config.jwt_secret, config.jwt_ttl_seconds);
        Self { pool, config: Arc::new(config), jwt: Arc::new(jwt) }
    }
}
