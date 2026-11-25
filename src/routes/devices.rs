use axum::{extract::State, Extension, Json};
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;
use crate::{auth::{require_role, Claims}, error::AppError, models::{DeviceResponse, RegisterDeviceRequest}, state::AppState, validation::validate};
#[derive(sqlx::FromRow, Serialize)] struct DeviceRow { id:Uuid, name:String, fleet_id:String, external_id:String, created_at:DateTime<Utc> }
impl From<DeviceRow> for DeviceResponse { fn from(d:DeviceRow)->Self { Self{id:d.id,name:d.name,fleet_id:d.fleet_id,external_id:d.external_id,created_at:d.created_at} } }
pub async fn register(State(s):State<AppState>, Extension(c):Extension<Claims>, Json(input):Json<RegisterDeviceRequest>) -> Result<Json<DeviceResponse>,AppError> {
    require_role(&c,&["admin"])?; validate(&input)?;
    let row=sqlx::query_as::<_,DeviceRow>("INSERT INTO devices(name,fleet_id,external_id,metadata) VALUES($1,$2,$3,$4::jsonb) RETURNING id,name,fleet_id,external_id,created_at")
        .bind(input.name).bind(input.fleet_id).bind(input.external_id).bind(input.metadata.unwrap_or_else(||serde_json::json!({})))
        .fetch_one(&s.pool).await?;
    Ok(Json(row.into()))
}
pub async fn list(State(s):State<AppState>, Extension(c):Extension<Claims>) -> Result<Json<Vec<DeviceResponse>>,AppError> {
    require_role(&c,&["admin","viewer","ingest"])?;
    let rows=sqlx::query_as::<_,DeviceRow>("SELECT id,name,fleet_id,external_id,created_at FROM devices ORDER BY created_at DESC LIMIT 500").fetch_all(&s.pool).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}
