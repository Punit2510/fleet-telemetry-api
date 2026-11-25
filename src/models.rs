use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email, length(max=254))] pub email: String,
    #[validate(length(min=12, max=128))] pub password: String,
}
#[derive(Debug, Serialize)]
pub struct TokenResponse { pub access_token: String, pub token_type: &'static str, pub expires_in: i64 }
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterDeviceRequest {
    #[validate(length(min=1, max=100))] pub name: String,
    #[validate(length(min=1, max=100))] pub fleet_id: String,
    #[validate(length(min=1, max=100))] pub external_id: String,
    #[validate(length(max=500))] pub metadata: Option<Value>,
}
#[derive(Debug, Serialize)]
pub struct DeviceResponse { pub id: Uuid, pub name: String, pub fleet_id: String, pub external_id: String, pub created_at: DateTime<Utc> }
#[derive(Debug, Deserialize, Validate)]
pub struct TelemetryBatch {
    #[validate(length(min=1))] pub events: Vec<TelemetryInput>,
}
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct TelemetryInput {
    pub device_id: Uuid,
    pub observed_at: DateTime<Utc>,
    #[validate(length(min=1, max=100))] pub metric: String,
    pub value: f64,
    #[validate(length(max=20))] pub unit: Option<String>,
    pub attributes: Option<Value>,
}
#[derive(Debug, Serialize)]
pub struct IngestResponse { pub accepted: usize }
#[derive(Debug, Deserialize)]
pub struct TelemetryQuery {
    pub device_id: Option<Uuid>,
    pub metric: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
}
#[derive(Debug, Serialize)]
pub struct TelemetryRecord {
    pub id: Uuid, pub device_id: Uuid, pub observed_at: DateTime<Utc>, pub metric: String,
    pub value: f64, pub unit: Option<String>, pub attributes: Option<Value>, pub received_at: DateTime<Utc>,
}
#[derive(Debug, Deserialize, Validate)]
pub struct CreateAlertRule {
    #[validate(length(min=1, max=100))] pub name: String,
    pub device_id: Option<Uuid>,
    #[validate(length(min=1, max=100))] pub metric: String,
    #[validate(length(min=1, max=3))] pub operator: String,
    pub threshold: f64,
    pub enabled: Option<bool>,
}
#[derive(Debug, Serialize)]
pub struct AlertRule {
    pub id: Uuid, pub name: String, pub device_id: Option<Uuid>, pub metric: String,
    pub operator: String, pub threshold: f64, pub enabled: bool, pub created_at: DateTime<Utc>,
}
