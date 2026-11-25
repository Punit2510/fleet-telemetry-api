use axum::{extract::{Query,State}, Extension, Json};
use chrono::{DateTime,Utc};
use serde_json::Value;
use uuid::Uuid;
use crate::{auth::{require_role,Claims},error::AppError,models::{IngestResponse,TelemetryBatch,TelemetryQuery,TelemetryRecord},state::AppState,validation::validate};

pub async fn ingest(State(s):State<AppState>,Extension(c):Extension<Claims>,Json(batch):Json<TelemetryBatch>)->Result<Json<IngestResponse>,AppError>{
    require_role(&c,&["admin","ingest"])?; validate(&batch)?;
    for event in &batch.events { validate(event)?; if !event.value.is_finite() { return Err(AppError::BadRequest("telemetry value must be finite".into())); } }
    if batch.events.len()>s.config.max_batch_size { return Err(AppError::PayloadTooLarge(format!("batch exceeds maximum of {} events",s.config.max_batch_size))); }
    let mut tx=s.pool.begin().await?;
    for e in &batch.events {
        let inserted=sqlx::query("INSERT INTO telemetry(device_id,observed_at,metric,value,unit,attributes) SELECT id,$2,$3,$4,$5,$6 FROM devices WHERE id=$1")
            .bind(e.device_id).bind(e.observed_at).bind(&e.metric).bind(e.value).bind(&e.unit)
            .bind(e.attributes.clone().unwrap_or(Value::Null)).execute(&mut *tx).await?;
        if inserted.rows_affected()==0 { return Err(AppError::BadRequest(format!("unknown device_id {}",e.device_id))); }
    }
    tx.commit().await?;
    Ok(Json(IngestResponse{accepted:batch.events.len()}))
}
pub async fn query(State(s):State<AppState>,Extension(c):Extension<Claims>,Query(q):Query<TelemetryQuery>)->Result<Json<Vec<TelemetryRecord>>,AppError>{
    require_role(&c,&["admin","viewer","ingest"])?;
    let limit=q.limit.unwrap_or(100).clamp(1,1000);
    let rows=sqlx::query_as::<_,TelemetryRow>(
        "SELECT id,device_id,observed_at,metric,value,unit,attributes,received_at FROM telemetry WHERE ($1::uuid IS NULL OR device_id=$1) AND ($2::text IS NULL OR metric=$2) AND ($3::timestamptz IS NULL OR observed_at >= $3) AND ($4::timestamptz IS NULL OR observed_at <= $4) ORDER BY observed_at DESC LIMIT $5"
    ).bind(q.device_id).bind(q.metric).bind(q.from).bind(q.to).bind(limit).fetch_all(&s.pool).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}
#[derive(sqlx::FromRow)] struct TelemetryRow{id:Uuid,device_id:Uuid,observed_at:DateTime<Utc>,metric:String,value:f64,unit:Option<String>,attributes:Option<Value>,received_at:DateTime<Utc>}
impl From<TelemetryRow> for TelemetryRecord{fn from(r:TelemetryRow)->Self{Self{id:r.id,device_id:r.device_id,observed_at:r.observed_at,metric:r.metric,value:r.value,unit:r.unit,attributes:r.attributes,received_at:r.received_at}}}
