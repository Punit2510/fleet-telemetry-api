use crate::{error::AppError, models::{AlertRule, CreateAlertRule}};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create(pool: &PgPool, input: CreateAlertRule) -> Result<AlertRule, AppError> {
    if !["gt", "gte", "lt", "lte", "eq"].contains(&input.operator.as_str()) {
        return Err(AppError::BadRequest("operator must be one of gt, gte, lt, lte, eq".into()));
    }
    let row = sqlx::query_as::<_, AlertRuleRow>(
        "INSERT INTO alert_rules (name, device_id, metric, operator, threshold, enabled) VALUES ($1,$2,$3,$4,$5,$6) RETURNING id,name,device_id,metric,operator,threshold,enabled,created_at"
    ).bind(input.name).bind(input.device_id).bind(input.metric).bind(input.operator)
     .bind(input.threshold).bind(input.enabled.unwrap_or(true)).fetch_one(pool).await?;
    Ok(row.into())
}
pub async fn list(pool: &PgPool) -> Result<Vec<AlertRule>, AppError> {
    let rows = sqlx::query_as::<_, AlertRuleRow>("SELECT id,name,device_id,metric,operator,threshold,enabled,created_at FROM alert_rules ORDER BY created_at DESC")
        .fetch_all(pool).await?;
    Ok(rows.into_iter().map(Into::into).collect())
}
#[derive(sqlx::FromRow)]
struct AlertRuleRow { id: Uuid, name: String, device_id: Option<Uuid>, metric: String, operator: String, threshold: f64, enabled: bool, created_at: chrono::DateTime<chrono::Utc> }
impl From<AlertRuleRow> for AlertRule {
    fn from(r: AlertRuleRow) -> Self { Self { id:r.id,name:r.name,device_id:r.device_id,metric:r.metric,operator:r.operator,threshold:r.threshold,enabled:r.enabled,created_at:r.created_at } }
}
