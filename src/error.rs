use reqwest::header::InvalidHeaderValue;
// src/error.rs
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("HTTP request failed: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("URL parsing failed: {0}")]
    UrlParseError(#[from] url::ParseError),

    #[error("JSON processing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("JSON deserialization failed: {0}")]
    JsonDeserializationFailed(String),

    #[error("Parse API error (code {code}): {error}")]
    ApiError { code: i32, error: String },

    #[error("Object not found: {0}")]
    ObjectNotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Invalid class name: {0}")]
    InvalidClassName(String),

    #[error("Invalid session token: {0}")]
    InvalidSessionToken(String),

    #[error("Session token is missing")]
    SessionTokenMissing,

    #[error("Master key required: {0}")]
    MasterKeyRequired(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Unexpected response: {0}")]
    UnexpectedResponse(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Unknown error: {0}")]
    Unknown(String),

    #[error("Operation forbidden: {0}")]
    OperationForbidden(String),

    #[error("Other Parse error (Code: {code}): {message}")]
    OtherParseError { code: u16, message: String },

    #[error("SDK error: {0}")]
    SdkError(String),

    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Invalid query: {0}")]
    InvalidQuery(String),

    #[error("Duplicate value: {0}")]
    DuplicateValue(String),

    #[error("Username taken: {0}")]
    UsernameTaken(String),

    #[error("Email taken: {0}")]
    EmailTaken(String),

    #[error("Internal server error: {0}")]
    InternalServerError(String),

    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(InvalidHeaderValue),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}

impl ParseError {
    /// Creates a `ParseError` from an HTTP status code and a JSON response body.
    pub(crate) fn from_response(status_code: u16, response_body: Value) -> Self {
        let error_code = response_body
            .get("code")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u16;
        let error_message = response_body
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown error")
            .to_string();

        match error_code {
            100 => ParseError::ConnectionFailed(format!("({}) {}", error_code, error_message)),
            101 => ParseError::ObjectNotFound(format!("({}) {}", error_code, error_message)), // Invalid username/password or object not found
            102 => ParseError::InvalidQuery(format!("({}) {}", error_code, error_message)),
            111 => ParseError::InvalidInput(format!(
                "Invalid field type: ({}) {}",
                error_code, error_message
            )),
            119 => ParseError::OperationForbidden(format!(
                "Missing master key for operation: ({}) {}",
                error_code, error_message
            )),
            137 => ParseError::DuplicateValue(format!("({}) {}", error_code, error_message)),
            202 => ParseError::UsernameTaken(format!("({}) {}", error_code, error_message)),
            203 => ParseError::EmailTaken(format!("({}) {}", error_code, error_message)),
            209 => ParseError::InvalidSessionToken(format!("({}) {}", error_code, error_message)),
            _ => {
                if status_code >= 500 {
                    ParseError::InternalServerError(format!(
                        "Server error (HTTP {}): ({}) {}",
                        status_code, error_code, error_message
                    ))
                } else if status_code == 401 || status_code == 403 {
                    ParseError::AuthenticationError(format!(
                        "Auth error (HTTP {}): ({}) {}",
                        status_code, error_code, error_message
                    ))
                } else if status_code == 404 {
                    ParseError::ObjectNotFound(format!(
                        "Not found (HTTP {}): ({}) {}",
                        status_code, error_code, error_message
                    ))
                } else {
                    ParseError::OtherParseError {
                        code: error_code,
                        message: error_message,
                    }
                }
            }
        }
    }
}
