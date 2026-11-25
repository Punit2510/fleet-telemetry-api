use axum::{http::{header, StatusCode}, response::{IntoResponse, Response}, Json};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FieldError { pub field: String, pub message: String }
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("request validation failed")] Validation(Vec<FieldError>),
    #[error("{0}")] BadRequest(String),
    #[error("{0}")] Unauthorized(&'static str),
    #[error("{0}")] Forbidden(&'static str),
    #[error("{0}")] NotFound(&'static str),
    #[error("{0}")] Conflict(String),
    #[error("{0}")] PayloadTooLarge(String),
    #[error("database error: {0}")] Database(sqlx::Error),
    #[error("internal error: {0}")] Internal(#[from] anyhow::Error),
}
#[derive(Serialize)]
struct ErrorBody { error: ErrorDetail }
#[derive(Serialize)]
struct ErrorDetail { code: &'static str, message: String, #[serde(skip_serializing_if="Option::is_none")] details: Option<Vec<FieldError>> }
impl AppError {
    pub fn status(&self) -> StatusCode { match self {
        Self::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY, Self::BadRequest(_) => StatusCode::BAD_REQUEST,
        Self::Unauthorized(_) => StatusCode::UNAUTHORIZED, Self::Forbidden(_) => StatusCode::FORBIDDEN,
        Self::NotFound(_) => StatusCode::NOT_FOUND, Self::Conflict(_) => StatusCode::CONFLICT,
        Self::PayloadTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE, Self::Database(_) | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }}
    fn code(&self) -> &'static str { match self {
        Self::Validation(_) => "validation_error", Self::BadRequest(_) => "bad_request",
        Self::Unauthorized(_) => "unauthorized", Self::Forbidden(_) => "forbidden",
        Self::NotFound(_) => "not_found", Self::Conflict(_) => "conflict",
        Self::PayloadTooLarge(_) => "payload_too_large", Self::Database(_) | Self::Internal(_) => "internal_error",
    }}
}
impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        match &e {
            sqlx::Error::RowNotFound => Self::NotFound("resource not found"),
            sqlx::Error::Database(d) => match d.code().as_deref() {
                Some("23505") => Self::Conflict("resource already exists".into()),
                Some("23503") => Self::BadRequest("referenced resource does not exist".into()),
                _ => Self::Database(e),
            },
            _ => Self::Database(e),
        }
    }
}
impl From<validator::ValidationErrors> for AppError {
    fn from(e: validator::ValidationErrors) -> Self {
        let mut details = Vec::new();
        for (field, errors) in e.field_errors() {
            for item in errors {
                details.push(FieldError { field: field.to_string(), message: item.message.as_ref().map(ToString::to_string).unwrap_or_else(|| format!("failed validation: {}", item.code)) });
            }
        }
        Self::Validation(details)
    }
}
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        let message = match &self {
            Self::Database(e) => { tracing::error!(error=%e, "database error"); "internal server error".to_string() },
            Self::Internal(e) => { tracing::error!(error=?e, "internal error"); "internal server error".to_string() },
            other => other.to_string(),
        };
        let details = match &self { Self::Validation(v) => Some(v.clone()), _ => None };
        let mut response = (status, Json(ErrorBody { error: ErrorDetail { code: self.code(), message, details } })).into_response();
        if status == StatusCode::UNAUTHORIZED { response.headers_mut().insert(header::WWW_AUTHENTICATE, header::HeaderValue::from_static("Bearer")); }
        response
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn maps_common_status_codes() {
        assert_eq!(AppError::NotFound("x").status(), StatusCode::NOT_FOUND);
        assert_eq!(AppError::Forbidden("x").status(), StatusCode::FORBIDDEN);
        assert_eq!(AppError::Conflict("x".into()).status(), StatusCode::CONFLICT);
    }
}
