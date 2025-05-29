use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::ParseError;
use crate::object::{deserialize_string_to_option_parse_date, deserialize_string_to_parse_date};
use crate::types::ParseDate;
use crate::Parse;

/// Represents a Parse Server Session object, detailing an active user session.
///
/// This struct includes standard fields for a session such as `objectId`, `sessionToken`,
/// `createdAt`, `updatedAt`, `expiresAt`, and information about the associated `user`.
/// It also captures `installationId` if the session originated from a specific installation,
/// and `createdWith` which describes how the session was initiated (e.g., login, signup).
///
/// The `user` field is a `serde_json::Value` because its content can vary. If the query
/// fetching the session includes `include=user`, this field will contain the full `ParseUser` object.
/// Otherwise, it might be a pointer or a more minimal representation.
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

    /// The user associated with this session. Can be a full `ParseUser` object if `include=user` is used.
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

/// Represents the successful response from a session update operation.
///
/// When a session is updated via the API (e.g., using `ParseSessionHandle::update_by_object_id`),
/// the server typically responds with the `updatedAt` timestamp of the modified session.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SessionUpdateResponse {
    /// The timestamp indicating when the session was last updated.
    pub updated_at: String,
}

/// Represents the response structure when fetching multiple sessions.
///
/// This struct is used to deserialize the JSON array of session objects returned by endpoints
/// like `/parse/sessions` when queried with the Master Key.
#[derive(Debug, Deserialize, Clone)]
pub struct GetAllSessionsResponse {
    /// A vector containing the [`ParseSession`](crate::session::ParseSession) objects retrieved.
    pub results: Vec<ParseSession>,
    // Optionally, add `count: Option<i64>` if count is requested and needed.
}

/// Provides methods for interacting with Parse Server sessions.
///
/// An instance of `ParseSessionHandle` is obtained by calling the [`session()`](crate::Parse::session)
/// method on a `Parse` instance. It allows for operations such as retrieving the current session's details,
/// fetching specific sessions by ID (Master Key required), deleting sessions (Master Key required),
/// updating sessions (Master Key required), and listing all sessions (Master Key required).
///
/// This handle operates in the context of the `Parse` it was created from, using its configuration
/// (server URL, app ID, keys) for API requests.
pub struct ParseSessionHandle<'a> {
    client: &'a Parse,
}

impl<'a> ParseSessionHandle<'a> {
    /// Creates a new `ParseSessionHandle`.
    ///
    /// This constructor is typically called by `Parse::session()`.
    ///
    /// # Arguments
    ///
    /// * `client`: A reference to the `Parse` instance that this handle will operate upon.
    pub fn new(client: &'a Parse) -> Self {
        ParseSessionHandle { client }
    }

    /// Retrieves the current user's session details.
    ///
    /// This method makes a GET request to the `/sessions/me` endpoint. It requires an active
    /// session token to be configured in the `Parse` (typically set after a successful
    /// login or signup). The server then returns the full details of the session associated
    /// with that token.
    ///
    /// If no session token is available in the client, this method will return a
    /// `ParseError::SessionTokenMissing` error without making a network request.
    ///
    /// # Returns
    ///
    /// A `Result` containing the [`ParseSession`](crate::session::ParseSession) object for the current session
    /// if successful, or a `ParseError` if the request fails (e.g., session token invalid/expired,
    /// network issue, or no session token present).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use parse_rs::{Parse, ParseError, user::LoginRequest};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), ParseError> {
    /// # let server_url = std::env::var("PARSE_SERVER_URL").unwrap_or_else(|_| "http://localhost:1338/parse".to_string());
    /// # let app_id = std::env::var("PARSE_APP_ID").unwrap_or_else(|_| "myAppId".to_string());
    /// # let mut client = Parse::new(&server_url, &app_id, None, None, None).await?;
    ///
    /// // Assume a user has logged in, so client.session_token() is Some(...)
    /// // let login_details = LoginRequest { username: "test_user", password: "password123" };
    /// // client.user().login(&login_details).await?;
    ///
    /// if client.is_authenticated() {
    ///     match client.session().me().await {
    ///         Ok(current_session) => {
    ///             println!("Current session details retrieved for user: {:?}", current_session.user);
    ///             println!("Session token: {}", current_session.session_token);
    ///             println!("Expires at: {:?}", current_session.expires_at);
    ///             // The session token in current_session should match client.session_token()
    ///             assert_eq!(Some(current_session.session_token.as_str()), client.session_token());
    ///         }
    ///         Err(e) => eprintln!("Failed to get current session details: {}", e),
    ///     }
    /// } else {
    ///     println!("No user is currently authenticated to get session details.");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    // Corresponds to GET /parse/sessions/me
    pub async fn me(&self) -> Result<ParseSession, ParseError> {
        if self.client.session_token.is_none() {
            return Err(ParseError::SessionTokenMissing);
        }
        // GET /sessions/me does not take a body and uses the client's current session token for auth.
        self.client
            ._request(Method::GET, "sessions/me", None::<&Value>, false, None) // false for use_master_key, None for explicit session_token (uses client's)
            .await
    }

    /// Retrieves a specific session by its `objectId`.
    ///
    /// This method makes a GET request to the `/sessions/:objectId` endpoint. It requires the
    /// Master Key to be configured in the `Parse` for authorization, as accessing arbitrary
    /// session objects is a privileged operation.
    ///
    /// # Arguments
    ///
    /// * `object_id`: A string slice representing the `objectId` of the session to retrieve.
    ///
    /// # Returns
    ///
    /// A `Result` containing the [`ParseSession`](crate::session::ParseSession) object if found and the Master Key is valid,
    /// or a `ParseError` if the session is not found, the Master Key is missing or invalid, or any
    /// other error occurs (e.g., network issue).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use parse_rs::{Parse, ParseError};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), ParseError> {
    /// # let server_url = std::env::var("PARSE_SERVER_URL").unwrap_or_else(|_| "http://localhost:1338/parse".to_string());
    /// # let app_id = std::env::var("PARSE_APP_ID").unwrap_or_else(|_| "myAppId".to_string());
    /// # let master_key = std::env::var("PARSE_MASTER_KEY").unwrap_or_else(|_| "myMasterKey".to_string());
    /// // Ensure the client is initialized with the Master Key for this operation
    /// let client = Parse::new(&server_url, &app_id, None, None, Some(&master_key)).await?;
    ///
    /// let session_object_id_to_fetch = "someValidSessionObjectId"; // Replace with an actual session objectId
    ///
    /// match client.session().get_by_object_id(session_object_id_to_fetch).await {
    ///     Ok(session) => {
    ///         println!("Successfully fetched session with objectId: {}", session.object_id);
    ///         println!("Associated user: {:?}", session.user);
    ///         println!("Session token: {}", session.session_token);
    ///     }
    ///     Err(e) => {
    ///         eprintln!("Failed to get session by objectId '{}': {}", session_object_id_to_fetch, e);
    ///         // This could be due to various reasons: session not found, master key invalid, network error, etc.
    ///         // e.g., if master key is missing or wrong, ParseError::MasterKeyMissingOrInvalid might be returned by the server (as unauthorized).
    ///         // e.g., if session_object_id_to_fetch does not exist, ParseError::ObjectNotFound might be returned.
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_by_object_id(&self, object_id: &str) -> Result<ParseSession, ParseError> {
        let endpoint = format!("sessions/{}", object_id);
        self.client
            ._request(Method::GET, &endpoint, None::<&Value>, true, None) // true for use_master_key
            .await
    }

    /// Deletes a specific session by its `objectId`.
    ///
    /// This method makes a DELETE request to the `/sessions/:objectId` endpoint. It requires the
    /// Master Key to be configured in the `Parse` for authorization, as deleting arbitrary
    /// session objects is a privileged operation. Successfully deleting a session effectively
    /// invalidates that session token, forcing the user to log in again.
    ///
    /// # Arguments
    ///
    /// * `object_id`: A string slice representing the `objectId` of the session to delete.
    ///
    /// # Returns
    ///
    /// A `Result` containing `()` (an empty tuple) if the session is successfully deleted.
    /// Returns a `ParseError` if the session is not found, the Master Key is missing or invalid,
    /// or any other error occurs (e.g., network issue).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use parse_rs::{Parse, ParseError};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), ParseError> {
    /// # let server_url = std::env::var("PARSE_SERVER_URL").unwrap_or_else(|_| "http://localhost:1338/parse".to_string());
    /// # let app_id = std::env::var("PARSE_APP_ID").unwrap_or_else(|_| "myAppId".to_string());
    /// # let master_key = std::env::var("PARSE_MASTER_KEY").unwrap_or_else(|_| "myMasterKey".to_string());
    /// // Ensure the client is initialized with the Master Key for this operation
    /// let client = Parse::new(&server_url, &app_id, None, None, Some(&master_key)).await?;
    ///
    /// let session_object_id_to_delete = "someSessionObjectIdToDelete"; // Replace with an actual session objectId
    ///
    /// match client.session().delete_by_object_id(session_object_id_to_delete).await {
    ///     Ok(_) => {
    ///         println!("Successfully deleted session with objectId: {}", session_object_id_to_delete);
    ///     }
    ///     Err(e) => {
    ///         eprintln!("Failed to delete session by objectId '{}': {}", session_object_id_to_delete, e);
    ///         // Common errors: ParseError::ObjectNotFound if the session doesn't exist,
    ///         // ParseError::MasterKeyMissingOrInvalid (or generic unauthorized) if master key is wrong.
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
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

    /// Updates a specific session by its `objectId`.
    ///
    /// This method makes a PUT request to the `/sessions/:objectId` endpoint, allowing modifications
    /// to the specified session object. It requires the Master Key to be configured in the `Parse`
    /// for authorization. The `session_data` argument should be a serializable struct or map
    /// containing the fields to be updated on the session object.
    ///
    /// Note: Not all fields on a session object are typically mutable. Consult the Parse Server
    /// documentation for details on which fields can be updated.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The type of the `session_data` argument. This type must implement `Serialize`, `Send`, and `Sync`.
    ///   It can be a custom struct representing the updatable fields or a `serde_json::Value` for dynamic updates.
    ///
    /// # Arguments
    ///
    /// * `object_id`: A string slice representing the `objectId` of the session to update.
    /// * `session_data`: A reference to the data containing the fields to update.
    ///
    /// # Returns
    ///
    /// A `Result` containing a [`SessionUpdateResponse`](crate::session::SessionUpdateResponse) (which typically includes the `updatedAt` timestamp)
    /// if the session is successfully updated. Returns a `ParseError` if the session is not found,
    /// the Master Key is missing or invalid, the update data is invalid, or any other error occurs.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use parse_rs::{Parse, ParseError, session::SessionUpdateResponse};
    /// use serde_json::json; // For creating a JSON value to update
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), ParseError> {
    /// # let server_url = std::env::var("PARSE_SERVER_URL").unwrap_or_else(|_| "http://localhost:1338/parse".to_string());
    /// # let app_id = std::env::var("PARSE_APP_ID").unwrap_or_else(|_| "myAppId".to_string());
    /// # let master_key = std::env::var("PARSE_MASTER_KEY").unwrap_or_else(|_| "myMasterKey".to_string());
    /// // Client must be initialized with the Master Key
    /// let client = Parse::new(&server_url, &app_id, None, None, Some(&master_key)).await?;
    ///
    /// let session_object_id_to_update = "someSessionObjectIdToUpdate"; // Replace with an actual session objectId
    ///
    /// // Example: Update a custom field on the session. Parse Server must be configured to allow this.
    /// // Let's assume we want to add/update a field `customData: { "notes": "privileged update" }`
    /// let update_payload = json!({
    ///     "customData": {
    ///         "notes": "privileged update by master key"
    ///     }
    /// });
    ///
    /// match client.session().update_by_object_id(session_object_id_to_update, &update_payload).await {
    ///     Ok(response) => {
    ///         println!(
    ///             "Successfully updated session '{}'. New updatedAt: {}",
    ///             session_object_id_to_update,
    ///             response.updated_at
    ///         );
    ///     }
    ///     Err(e) => {
    ///         eprintln!(
    ///             "Failed to update session by objectId '{}': {}",
    ///             session_object_id_to_update, e
    ///         );
    ///         // Common errors: ParseError::ObjectNotFound, ParseError::MasterKeyMissingOrInvalid, or invalid update payload.
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
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

    /// Retrieves multiple sessions, optionally filtered and paginated using a query string.
    ///
    /// This method makes a GET request to the `/sessions` endpoint. It requires the Master Key
    /// to be configured in the `Parse` for authorization, as listing all sessions is a
    /// highly privileged operation.
    ///
    /// The `query_string` argument allows for server-side filtering, pagination, ordering, and
    /// inclusion of related data (like the `user` object via `include=user`).
    ///
    /// # Arguments
    ///
    /// * `query_string`: An optional string slice representing the URL-encoded query parameters.
    ///   For example: `"limit=10&skip=20&include=user&where={\"user\":{\"$inQuery\":{\"where\":{\"username\":\"test_user\"},\"className\":\"_User\"}}}"`
    ///   If `None`, all sessions (up to the server's default limit) are requested.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Vec<ParseSession>` if the request is successful. The vector will
    /// contain the session objects matching the query. Returns a `ParseError` if the Master Key
    /// is missing or invalid, the query string is malformed, or any other error occurs.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use parse_rs::{Parse, ParseError};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), ParseError> {
    /// # let server_url = std::env::var("PARSE_SERVER_URL").unwrap_or_else(|_| "http://localhost:1338/parse".to_string());
    /// # let app_id = std::env::var("PARSE_APP_ID").unwrap_or_else(|_| "myAppId".to_string());
    /// # let master_key = std::env::var("PARSE_MASTER_KEY").unwrap_or_else(|_| "myMasterKey".to_string());
    /// // Client must be initialized with the Master Key
    /// let client = Parse::new(&server_url, &app_id, None, None, Some(&master_key)).await?;
    ///
    /// // Example 1: Get all sessions (respecting server default limit) and include user data
    /// match client.session().get_all_sessions(Some("include=user")).await {
    ///     Ok(sessions) => {
    ///         println!("Successfully retrieved {} sessions (with user data):", sessions.len());
    ///         for session in sessions {
    ///             println!("  Session ID: {}, User: {:?}", session.object_id, session.user);
    ///         }
    ///     }
    ///     Err(e) => eprintln!("Failed to get all sessions with user data: {}", e),
    /// }
    ///
    /// // Example 2: Get the first 5 sessions, ordered by creation date descending
    /// let query_params = "limit=5&order=-createdAt";
    /// match client.session().get_all_sessions(Some(query_params)).await {
    ///     Ok(sessions) => {
    ///         println!("\nSuccessfully retrieved {} sessions (first 5, newest first):", sessions.len());
    ///         for session in sessions {
    ///             println!("  Session ID: {}, Created At: {:?}", session.object_id, session.created_at);
    ///         }
    ///     }
    ///     Err(e) => eprintln!("Failed to get paginated/ordered sessions: {}", e),
    /// }
    ///
    /// // Example 3: Get sessions without any query parameters (server defaults apply)
    /// match client.session().get_all_sessions(None).await {
    ///     Ok(sessions) => {
    ///         println!("\nSuccessfully retrieved {} sessions (server defaults):", sessions.len());
    ///     }
    ///     Err(e) => eprintln!("Failed to get sessions with no query: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
