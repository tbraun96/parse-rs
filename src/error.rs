// src/error.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("API error: {code} - {message}")]
    ApiError {
        code: u16,
        message: String,
    },
    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("URL parsing error: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Invalid session token")]
    InvalidSessionToken,
    #[error("Operation forbidden: {0}")]
    Forbidden(String),
    #[error("Resource not found: {0}")]
    NotFound(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}
