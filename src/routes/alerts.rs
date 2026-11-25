use axum::{extract::State,Extension,Json};
use crate::{alerts,error::AppError,auth::{Claims,require_role},models::{AlertRule,CreateAlertRule},state::AppState,validation::validate};
pub async fn create(State(s):State<AppState>,Extension(c):Extension<Claims>,Json(input):Json<CreateAlertRule>)->Result<Json<AlertRule>,AppError>{
    require_role(&c,&["admin"])?; validate(&input)?;
    if !input.threshold.is_finite(){return Err(AppError::BadRequest("threshold must be a finite number".into()));}
    Ok(Json(alerts::create(&s.pool,input).await?))
}
pub async fn list(State(s):State<AppState>,Extension(c):Extension<Claims>)->Result<Json<Vec<AlertRule>>,AppError>{
    require_role(&c,&["admin","viewer"])?; Ok(Json(alerts::list(&s.pool).await?))
}
