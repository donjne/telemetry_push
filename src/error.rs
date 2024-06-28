use std::fmt;
use argon2::password_hash::Error as ArgonError; // Rename as necessary for your imports
use sqlx::Error as SqlxError; // Rename as necessary for your imports

#[derive(Debug)]
pub enum CustomError {
    Argon2Error(ArgonError),
    SqlxError(SqlxError),
    OtherError(String),
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CustomError::Argon2Error(e) => write!(f, "Argon2 error: {}", e),
            CustomError::SqlxError(e) => write!(f, "SQLx error: {}", e),
            CustomError::OtherError(e) => write!(f, "Other error: {}", e),
        }
    }
}

impl std::error::Error for CustomError {}

// Optional: Implement From trait for easier error conversion
impl From<ArgonError> for CustomError {
    fn from(err: ArgonError) -> Self {
        CustomError::Argon2Error(err)
    }
}

impl From<SqlxError> for CustomError {
    fn from(err: SqlxError) -> Self {
        CustomError::SqlxError(err)
    }
}

// Add more conversions as needed for other error types
