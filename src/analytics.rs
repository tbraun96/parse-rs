// src/analytics.rs
use crate::client::Parse;
use crate::error::ParseError;
use reqwest::Method;
use serde_json::Value;

impl Parse {
    /// Tracks a custom event with optional dimensions.
    ///
    /// # Arguments
    /// * `event_name`: The name of the event to track (e.g., "ButtonClicked", "ItemPurchased").
    /// * `dimensions`: Optional key-value pairs to associate with the event.
    ///
    /// # Returns
    /// A `Result` indicating success or a `ParseError`.
    ///
    /// This operation typically requires the Master Key, JavaScript Key, or REST API Key.
    pub async fn track_event(
        &self,
        event_name: &str,
        dimensions: Option<Value>,
    ) -> Result<(), ParseError> {
        if event_name.is_empty() {
            return Err(ParseError::InvalidInput(
                "Event name cannot be empty.".to_string(),
            ));
        }

        let endpoint = format!("events/{}", event_name);

        // The server expects an empty JSON object {} if dimensions are None.
        let body = dimensions.unwrap_or_else(|| serde_json::json!({}));

        let use_master_key = self.master_key.is_some();
        // Analytics events are often not tied to a specific user session.
        // The _request method will use JS or REST key if master_key is None and session_token is None.
        let session_token_to_use = None;

        // For analytics events, Parse Server responds with an empty JSON object {} and HTTP 200 OK on success.
        // We don't need to deserialize a specific response body, just ensure the request was successful.
        let _response_value: Value = self
            ._request(
                Method::POST,
                &endpoint,
                Some(&body),
                use_master_key,
                session_token_to_use,
            )
            .await?;

        // If _request didn't return an error (e.g., non-2xx status), we consider it a success.
        // The _request method itself handles non-successful HTTP status codes by returning ParseError.
        Ok(())
    }
}
