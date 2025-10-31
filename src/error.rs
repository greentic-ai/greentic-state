use greentic_types::{ErrorCode, GreenticError};
use serde_json::Error as SerdeError;
use std::fmt::Display;

/// Builds an `InvalidInput` error with the provided message.
pub fn invalid_input(message: impl Into<String>) -> GreenticError {
    GreenticError::new(ErrorCode::InvalidInput, message)
}

/// Builds an `Internal` error for unexpected backend failures.
pub fn internal(message: impl Into<String>) -> GreenticError {
    GreenticError::new(ErrorCode::Internal, message)
}

/// Builds an `Unavailable` error for backend outages.
pub fn unavailable(message: impl Into<String>) -> GreenticError {
    GreenticError::new(ErrorCode::Unavailable, message)
}

/// Wraps a `serde_json` error as `InvalidInput`.
pub fn from_serde(err: SerdeError) -> GreenticError {
    invalid_input(err.to_string())
}

/// Attaches additional context to an error message.
pub fn with_context(err: impl Display, context: impl Into<String>) -> GreenticError {
    internal(format!("{}: {}", context.into(), err))
}

#[cfg(feature = "redis")]
/// Wraps a Redis error as `Unavailable`.
pub fn from_redis(err: redis::RedisError, context: impl Into<String>) -> GreenticError {
    unavailable(format!("{}: {}", context.into(), err))
}
