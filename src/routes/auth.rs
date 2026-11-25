use axum::{extract::State, Json};
use crate::{auth::verify_password, error::AppError, models::{LoginRequest, TokenResponse}, state::AppState, validation::validate};
#[derive(sqlx::FromRow)] struct User { id: uuid::Uuid, email: String, password_hash: String, role: String }
pub async fn login(State(state): State<AppState>, Json(input): Json<LoginRequest>) -> Result<Json<TokenResponse>, AppError> {
    validate(&input)?;
    let user = sqlx::query_as::<_, User>("SELECT id,email,password_hash,role FROM users WHERE email=$1")
        .bind(&input.email).fetch_optional(&state.pool).await?;
    let valid = user.as_ref().is_some_and(|u| verify_password(&input.password, &u.password_hash));
    if !valid { return Err(AppError::Unauthorized("invalid email or password")); }
    let user = user.unwrap();
    let token = state.jwt.issue(&user.id.to_string(), &user.email, &user.role)?;
    Ok(Json(TokenResponse { access_token:token, token_type:"Bearer", expires_in:state.config.jwt_ttl_seconds }))
}
