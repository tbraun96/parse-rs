// use crate::acl::ParseACL; // Unused
use crate::error::ParseError;

use reqwest::{Method, Response as HttpResponse};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

impl crate::Parse {
    // New internal helper to send a pre-built request and process its response.
    pub(crate) async fn _send_and_process_response<R: DeserializeOwned + Send + 'static>(
        &self, // Keep &self for potential future use, though not strictly needed by current logic
        response: HttpResponse, // Changed from request_builder
        _endpoint_context: &str, // Added for logging/error context
    ) -> Result<R, ParseError> {
        let status = response.status();
        let response_url = response.url().to_string(); // For logging

        // Try to get the body as text first for logging, then consume for JSON
        let response_text = response.text().await.map_err(ParseError::ReqwestError)?;

        if status.is_success() {
            if response_text.is_empty() || response_text == "{}" {
                // Handle cases where R is (), expecting no content or empty object
                if std::any::TypeId::of::<R>() == std::any::TypeId::of::<()>() {
                    // Attempt to deserialize from "null" as a convention for empty successful responses
                    // This allows `()` to be a valid response type for 204 No Content or empty {} body.
                    return serde_json::from_str("null").map_err(ParseError::JsonError);
                }
            }
            // Attempt to deserialize the response body
            serde_json::from_str::<R>(&response_text).map_err(|e| {
                log::error!(
                    "JSON Deserialization failed for successful response from '{}'. Status: {}. Error: {}. Body: {}",
                    response_url,
                    status,
                    e,
                    &response_text // Log the problematic text
                );
                ParseError::JsonDeserializationFailed(format!(
                    "Failed to deserialize successful response from '{}': {}. Body: {}",
                    response_url, e, &response_text
                ))
            })
        } else {
            // Attempt to parse the error response body as JSON
            let parsed_body: Value = match serde_json::from_str(&response_text) {
                Ok(json_val) => json_val,
                Err(_) => {
                    // If parsing the error body as JSON fails, create a generic error
                    log::warn!(
                        "Failed to parse error response body as JSON from '{}'. Status: {}. Body: {}",
                        response_url, status, &response_text
                    );
                    Value::Object(serde_json::Map::from_iter(vec![
                        (
                            "error".to_string(),
                            Value::String(format!("HTTP Error {} with non-JSON body", status)),
                        ),
                        (
                            "body_snippet".to_string(),
                            Value::String(response_text.chars().take(100).collect()),
                        ),
                    ]))
                }
            };
            Err(ParseError::from_response(status.as_u16(), parsed_body))
        }
    }

    // Public HTTP method wrappers
    pub async fn get<R: DeserializeOwned + Send + 'static>(
        &self,
        endpoint: &str,
    ) -> Result<R, ParseError> {
        self._request(Method::GET, endpoint, None::<&Value>, false, None)
            .await
    }

    pub async fn post<T: Serialize + Send + Sync, R: DeserializeOwned + Send + 'static>(
        &self,
        endpoint: &str,
        data: &T,
    ) -> Result<R, ParseError> {
        self._request(Method::POST, endpoint, Some(data), false, None)
            .await
    }

    pub async fn put<T: Serialize + Send + Sync, R: DeserializeOwned + Send + 'static>(
        &self,
        endpoint: &str,
        data: &T,
    ) -> Result<R, ParseError> {
        self._request(Method::PUT, endpoint, Some(data), false, None)
            .await
    }

    pub async fn delete<R: DeserializeOwned + Send + 'static>(
        &self,
        endpoint: &str,
    ) -> Result<R, ParseError> {
        self._request(Method::DELETE, endpoint, None::<&Value>, false, None)
            .await
    }
}
