use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::ParseError;
use crate::object::{deserialize_string_to_option_parse_date, deserialize_string_to_parse_date};
use crate::types::ParseDate;
use crate::Parse;

/// Represents a Parse Session object.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParseSession {
    #[serde(rename = "objectId")]
    pub object_id: String,

    #[serde(deserialize_with = "deserialize_string_to_parse_date")]
    #[serde(rename = "createdAt")]
    pub created_at: ParseDate,

    #[serde(default)] // Default will be Option::None
    #[serde(deserialize_with = "deserialize_string_to_option_parse_date")]
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<ParseDate>,

    pub user: Value, // When 'include=user' is used, this will be the full User object

    #[serde(rename = "sessionToken")]
    pub session_token: String,

    #[serde(rename = "installationId")]
    pub installation_id: Option<String>,

    #[serde(rename = "expiresAt")]
    #[serde(default)] // Default will be Option::None
    pub expires_at: Option<ParseDate>,

    pub restricted: Option<bool>, // Typically false for normal sessions, true for restricted ones

    #[serde(rename = "createdWith")]
    pub created_with: Option<Value>, // e.g., {"action": "login", "authProvider": "password"}

    // Catch all for other fields
    #[serde(flatten)]
    pub other_fields: std::collections::HashMap<String, Value>,
}

/// Response from a session update operation.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SessionUpdateResponse {
    pub updated_at: String,
}

/// Response from a get all sessions operation.
#[derive(Debug, Deserialize, Clone)]
pub struct GetAllSessionsResponse {
    pub results: Vec<ParseSession>,
    // Optionally, add `count: Option<i64>` if count is requested and needed.
}

/// Provides methods for interacting with Parse Server sessions.
pub struct ParseSessionHandle<'a> {
    client: &'a Parse,
}

impl<'a> ParseSessionHandle<'a> {
    pub fn new(client: &'a Parse) -> Self {
        ParseSessionHandle { client }
    }

    /// Retrieves the current user's session details.
    /// Requires an active session token to be configured in the client.
    /// Corresponds to GET /parse/sessions/me
    pub async fn me(&self) -> Result<ParseSession, ParseError> {
        if self.client.session_token.is_none() {
            return Err(ParseError::SessionTokenMissing);
        }
        // GET /sessions/me does not take a body and uses the client's current session token for auth.
        self.client
            ._request(Method::GET, "sessions/me", None::<&Value>, false, None) // false for use_master_key, None for explicit session_token (uses client's)
            .await
    }

    /// Retrieves a specific session by its objectId.
    /// This operation requires the Master Key.
    pub async fn get_by_object_id(&self, object_id: &str) -> Result<ParseSession, ParseError> {
        let endpoint = format!("sessions/{}", object_id);
        self.client
            ._request(Method::GET, &endpoint, None::<&Value>, true, None) // true for use_master_key
            .await
    }

    /// Deletes a specific session by its objectId.
    /// This operation requires the Master Key.
    pub async fn delete_by_object_id(&self, object_id: &str) -> Result<(), ParseError> {
        let endpoint = format!("sessions/{}", object_id);
        // Expect serde_json::Value to consume the empty {} response, then map to Ok(()).
        // The master key is required for this operation.
        let _: Value = self
            .client
            ._request(Method::DELETE, &endpoint, None::<&Value>, true, None) // true for use_master_key
            .await?;
        Ok(())
    }

    /// Updates a specific session by its objectId.
    /// This operation requires the Master Key.
    /// The `session_data` should be a serializable struct representing the fields to update.
    pub async fn update_by_object_id<T: Serialize + Send + Sync>(
        &self,
        object_id: &str,
        session_data: &T,
    ) -> Result<SessionUpdateResponse, ParseError> {
        let endpoint = format!("sessions/{}", object_id);
        self.client
            ._request(Method::PUT, &endpoint, Some(session_data), true, None) // true for use_master_key
            .await
    }

    /// Retrieves multiple sessions, optionally filtered by a query string.
    /// This operation requires the Master Key.
    /// The `query_string` should be the URL-encoded string of parameters (e.g., "limit=10&include=user").
    pub async fn get_all_sessions(
        &self,
        query_string: Option<&str>,
    ) -> Result<Vec<ParseSession>, ParseError> {
        #[derive(Deserialize, Debug)]
        struct SessionsResponse<T> {
            results: Vec<T>,
        }

        let endpoint = match query_string {
            Some(qs) => format!("sessions?{}", qs),
            None => "sessions".to_string(),
        };
        let response: SessionsResponse<ParseSession> = self
            .client
            ._request(Method::GET, &endpoint, None::<&Value>, true, None) // true for use_master_key
            .await?;
        Ok(response.results)
    }
}
