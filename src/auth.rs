use axum::{body::Body, extract::State, http::{header, Request}, middleware::Next, response::Response};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use crate::{error::AppError, state::AppState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims { pub sub: String, pub email: String, pub role: String, pub exp: usize, pub iat: usize }
pub struct JwtKeys { encoding: EncodingKey, decoding: DecodingKey, ttl: i64 }
impl JwtKeys {
    pub fn new(secret: &str, ttl: i64) -> Self {
        Self { encoding: EncodingKey::from_secret(secret.as_bytes()), decoding: DecodingKey::from_secret(secret.as_bytes()), ttl }
    }
    pub fn issue(&self, sub: &str, email: &str, role: &str) -> Result<String, AppError> {
        let now = Utc::now();
        let claims = Claims { sub: sub.into(), email: email.into(), role: role.into(), iat: now.timestamp() as usize, exp: (now + Duration::seconds(self.ttl)).timestamp() as usize };
        encode(&Header::default(), &claims, &self.encoding).map_err(|e| AppError::Internal(e.into()))
    }
    pub fn verify(&self, token: &str) -> Result<Claims, AppError> {
        decode::<Claims>(token, &self.decoding, &Validation::default()).map(|d| d.claims).map_err(|_| AppError::Unauthorized("invalid or expired token"))
    }
}
pub fn dummy_hash() -> String { hash("not-a-real-password", DEFAULT_COST).unwrap_or_default() }
pub async fn require_auth(State(state): State<AppState>, mut request: Request<Body>, next: Next) -> Result<Response, AppError> {
    let value = request.headers().get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()).unwrap_or("");
    let token = value.strip_prefix("Bearer ").ok_or(AppError::Unauthorized("missing bearer token"))?;
    let claims = state.jwt.verify(token)?;
    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}
pub fn require_role(claims: &Claims, allowed: &[&str]) -> Result<(), AppError> {
    if allowed.iter().any(|r| *r == claims.role) { Ok(()) } else { Err(AppError::Forbidden("insufficient role")) }
}
pub fn verify_password(password: &str, encoded: &str) -> bool { verify(password, encoded).unwrap_or(false) }
pub fn hash_password(password: &str) -> Result<String, AppError> { hash(password, DEFAULT_COST).map_err(|e| AppError::Internal(e.into())) }
