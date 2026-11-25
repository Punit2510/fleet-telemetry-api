use axum::{extract::State, Json};
use serde::Serialize;
use crate::{error::AppError, state::AppState};
#[derive(Serialize)] pub struct Health { status: &'static str, database: &'static str }
pub async fn health(State(state): State<AppState>) -> Result<Json<Health>, AppError> {
    sqlx::query("SELECT 1").execute(&state.pool).await?;
    Ok(Json(Health { status:"ok", database:"ok" }))
}
