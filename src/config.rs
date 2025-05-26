use crate::client::UpdateConfigResponse;
use crate::ParseError;
use reqwest::Method;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

/// Represents the Parse Server configuration parameters.
/// The server returns parameters nested under a "params" key.
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct ParseConfig {
    /// A map of parameter names to their values.
    pub params: HashMap<String, Value>,
}

// We might add helper methods to ParseConfig later, e.g., to get a specific typed parameter.
impl ParseConfig {
    /// Retrieves a specific parameter by name and attempts to deserialize it into the requested type.
    ///
    /// # Arguments
    /// * `key`: The name of the parameter to retrieve.
    ///
    /// # Returns
    /// An `Option<T>` containing the deserialized value if the key exists and deserialization is successful,
    /// otherwise `None`.
    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.params
            .get(key)
            .and_then(|value| serde_json::from_value(value.clone()).ok())
    }
}

impl crate::Parse {
    // Config management methods

    /// Retrieves the Parse Server configuration.
    ///
    /// This operation requires the Master Key.
    ///
    /// # Returns
    /// A `Result` containing the `ParseConfig` or a `ParseError`.
    pub async fn get_config(&self) -> Result<ParseConfig, ParseError> {
        if self.master_key.is_none() {
            return Err(ParseError::MasterKeyRequired(
                "Master key is required to get server configuration.".to_string(),
            ));
        }

        let endpoint = "config";
        self._request(
            Method::GET,
            endpoint,
            None::<&Value>, // No body for GET
            true,           // Use master key
            None,           // No session token needed when using master key
        )
        .await
    }

    /// Updates the Parse Server configuration parameters.
    ///
    /// This operation requires the Master Key.
    /// The `params_to_update` should contain only the parameters you wish to change.
    ///
    /// # Arguments
    /// * `params_to_update`: A `HashMap<String, Value>` of parameters to update.
    ///
    /// # Returns
    /// A `Result` indicating success (typically an empty successful response or a confirmation)
    /// or a `ParseError`.
    /// The Parse Server responds with `{"result": true}` on successful update.
    pub async fn update_config(
        &self,
        params_to_update: &HashMap<String, Value>,
    ) -> Result<UpdateConfigResponse, ParseError> {
        if self.master_key.is_none() {
            return Err(ParseError::MasterKeyRequired(
                "Master key is required to update server configuration.".to_string(),
            ));
        }
        if params_to_update.is_empty() {
            return Err(ParseError::InvalidInput(
                "params_to_update cannot be empty for update_config.".to_string(),
            ));
        }

        let endpoint = "config";
        // The body should be wrapped: {"params": params_to_update}
        let body = serde_json::json!({ "params": params_to_update });

        self._request(
            Method::PUT,
            endpoint,
            Some(&body), // Pass the wrapped body
            true,        // Use master key
            None,        // No session token needed
        )
        .await
    }
}
