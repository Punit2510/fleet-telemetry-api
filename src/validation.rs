use crate::error::AppError;
use validator::Validate;
pub fn validate<T: Validate>(value: &T) -> Result<(), AppError> { value.validate().map_err(AppError::from) }
