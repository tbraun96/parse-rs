// src/client.rs

use crate::error::ParseError;
use crate::object::ParseObject;
use crate::schema::{GetAllSchemasResponse, ParseSchema};
use crate::user::ParseUserHandle;
use crate::FileField;
use crate::ParseCloud;

use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use reqwest::{Client, Method, Url};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

/// Specifies the type of authentication credentials to be used for an API request.
///
/// This enum helps determine which keys or tokens are prioritized when constructing
/// the headers for a request to the Parse Server.
pub enum AuthType {
    /// Use the current session token. This is typically obtained after a user logs in.
    /// If no session token is available, the request might fail or use other credentials
    /// based on client configuration.
    SessionToken,
    /// Use the Master Key. This key bypasses all ACLs and Class-Level Permissions.
    /// It should be used sparingly and kept secure.
    MasterKey,
    /// Use the REST API Key. If the REST API Key is not configured on the client,
    /// it may fall back to using the JavaScript Key if that is configured.
    /// This key is typically used for general API access from trusted server environments.
    RestApiKey,
    /// No specific authentication to be actively chosen for this request.
    /// The request will rely on the default headers configured in the `Parse`
    /// (e.g., Application ID, and potentially a pre-configured JavaScript Key or REST API Key if no Master Key was set globally).
    /// This is suitable for operations that don't require user context or elevated privileges,
    /// such as public data queries or user signup/login endpoints themselves.
    NoAuth,
}

/// The main client for interacting with a Parse Server instance.
///
/// `Parse` handles the configuration of server connection details (URL, Application ID, API keys)
/// and provides methods for making authenticated or unauthenticated requests to various Parse Server endpoints.
/// It manages session tokens for authenticated users and uses an underlying `reqwest::Client` for HTTP communication.
///
/// Most operations are performed by calling methods directly on `Parse` or by obtaining specialized
/// handles (like `ParseUserHandle`, `ParseSessionHandle`, `ParseCloud`) through methods on this client.
///
/// # Initialization
///
/// A `Parse` is typically created using the [`Parse::new()`] method, providing the server URL,
/// Application ID, and any relevant API keys (JavaScript, REST, Master).
///
/// ```rust,no_run
/// use parse_rs::Parse;
/// # use parse_rs::ParseError;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), ParseError> {
/// let server_url = "http://localhost:1338/parse";
/// let app_id = "myAppId";
/// let master_key = "myMasterKey";
///
/// // Create a client instance with Master Key
/// let mut client = Parse::new(
///     server_url,
///     app_id,
///     None, // javascript_key
///     None, // rest_api_key
///     Some(master_key), // master_key
/// ).await?;
///
/// // Client is now ready to be used
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Parse {
    pub server_url: String, // Changed from Url to String
    pub(crate) app_id: String,
    #[allow(dead_code)] // Not used by current auth features
    pub(crate) javascript_key: Option<String>,
    pub(crate) rest_api_key: Option<String>,
    pub(crate) master_key: Option<String>,
    pub(crate) http_client: Client, // Updated to use alias
    pub(crate) session_token: Option<String>,
}

impl Parse {
    /// Creates a new `Parse` instance.
    ///
    /// This constructor initializes the client with the necessary credentials and configuration
    /// to communicate with your Parse Server.
    ///
    /// # Arguments
    ///
    /// * `server_url`: The base URL of your Parse Server (e.g., `"http://localhost:1338/parse"`).
    ///   The client will attempt to normalize this URL (e.g., ensure scheme, remove trailing `/parse` if present
    ///   to derive the true server base for constructing endpoint paths).
    /// * `app_id`: Your Parse Application ID. This is a required header for all requests.
    /// * `javascript_key`: Optional. Your Parse JavaScript Key. If provided and `master_key` is not,
    ///   this key will be included in requests by default, unless overridden by a session token or explicit master key usage.
    /// * `rest_api_key`: Optional. Your Parse REST API Key. If provided and both `master_key` and `javascript_key` are not,
    ///   this key will be included in requests by default. It's generally preferred over the JavaScript Key for server-to-server communication.
    /// * `master_key`: Optional. Your Parse Master Key. If provided, this key will be included in requests by default,
    ///   granting unrestricted access. Use with caution. It supersedes other keys for default authentication if present.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `Parse` instance if successful, or a `ParseError` if
    /// configuration is invalid (e.g., invalid URL, invalid header values).
    ///
    /// # Key Precedence for Default Headers
    /// When the client makes requests, the authentication key used in the default headers (when no session token is active
    /// and `use_master_key` is not explicitly set for an operation) follows this precedence:
    /// 1. Master Key (if provided at initialization)
    /// 2. JavaScript Key (if provided and Master Key is not)
    /// 3. REST API Key (if provided and neither Master Key nor JavaScript Key are)
    ///
    /// A session token, once set (e.g., after login), will typically take precedence over these default keys for most operations.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use parse_rs::Parse;
    /// # use parse_rs::ParseError;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), ParseError> {
    /// let server_url = "http://localhost:1338/parse";
    /// let app_id = "myAppId";
    /// let js_key = "myJavascriptKey";
    ///
    /// // Initialize with JavaScript Key
    /// let mut client_with_js_key = Parse::new(
    ///     server_url,
    ///     app_id,
    ///     Some(js_key),
    ///     None, // rest_api_key
    ///     None, // master_key
    /// ).await?;
    ///
    /// // Initialize with Master Key (will take precedence for default auth)
    /// let master_key = "myMasterKey";
    /// let mut client_with_master_key = Parse::new(
    ///     server_url,
    ///     app_id,
    ///     Some(js_key), // JS key is also provided
    ///     None,         // REST API key
    ///     Some(master_key), // Master key will be used by default
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(
        server_url: &str,
        app_id: &str,
        javascript_key: Option<&str>,
        rest_api_key: Option<&str>,
        master_key: Option<&str>,
    ) -> Result<Self, ParseError> {
        let mut temp_url_string = server_url.to_string();

        // Ensure scheme is present
        if !temp_url_string.starts_with("http://") && !temp_url_string.starts_with("https://") {
            temp_url_string = format!("http://{}", temp_url_string);
        }

        let parsed_server_url = Url::parse(&temp_url_string)?;

        if parsed_server_url.cannot_be_a_base() {
            return Err(ParseError::SdkError(format!(
                "The server_url '{}' (after ensuring scheme) resolved to '{}', which cannot be a base URL. Please provide a full base URL (e.g., http://localhost:1337/parse).",
                server_url, parsed_server_url
            )));
        }

        let mut default_headers = HeaderMap::new();
        default_headers.insert(
            "X-Parse-Application-Id",
            HeaderValue::from_str(app_id).map_err(ParseError::InvalidHeaderValue)?,
        );

        if let Some(mk_str) = master_key {
            default_headers.insert(
                "X-Parse-Master-Key",
                HeaderValue::from_str(mk_str).map_err(ParseError::InvalidHeaderValue)?,
            );
        } else if let Some(js_key_str) = javascript_key {
            default_headers.insert(
                "X-Parse-Javascript-Key",
                HeaderValue::from_str(js_key_str).map_err(ParseError::InvalidHeaderValue)?,
            );
        } else if let Some(rk_str) = rest_api_key {
            default_headers.insert(
                "X-Parse-REST-API-Key",
                HeaderValue::from_str(rk_str).map_err(ParseError::InvalidHeaderValue)?,
            );
        }

        let http_client = Client::builder() // Updated to use alias
            .default_headers(default_headers)
            .build()
            .map_err(ParseError::ReqwestError)?;

        let mut final_server_url = parsed_server_url.as_str().trim_end_matches('/').to_string();

        // If the URL ends with /parse, strip it to get the true base server URL.
        // This makes the client resilient to PARSE_SERVER_URL being http://host/parse or http://host.
        if final_server_url.ends_with("/parse") {
            final_server_url.truncate(final_server_url.len() - "/parse".len());
        }
        // Ensure it's not empty after stripping (e.g. if PARSE_SERVER_URL was just "/parse")
        if final_server_url.is_empty() && parsed_server_url.scheme() == "http"
            || parsed_server_url.scheme() == "https"
        {
            // This case is unlikely if original URL was valid, but as a safeguard.
            // Reconstruct from scheme and host if available, or error.
            if let Some(host_str) = parsed_server_url.host_str() {
                final_server_url = format!("{}://{}", parsed_server_url.scheme(), host_str);
                if let Some(port) = parsed_server_url.port() {
                    final_server_url.push_str(&format!(":{}", port));
                }
            } else {
                return Err(ParseError::SdkError("Server URL became empty after stripping /parse and could not be reconstructed.".to_string()));
            }
        }

        log::debug!(
            "Parse initialized with base server_url: {}",
            final_server_url
        );

        Ok(Self {
            server_url: final_server_url,
            app_id: app_id.to_string(),
            javascript_key: javascript_key.map(|s| s.to_string()),
            rest_api_key: rest_api_key.map(|s| s.to_string()),
            master_key: master_key.map(|s| s.to_string()),
            http_client,
            session_token: None,
        })
    }

    // Internal method to set or clear the session token.
    pub(crate) fn _set_session_token(&mut self, token: Option<String>) {
        self.session_token = token;
    }

    /// Returns the current session token, if one is set on the client.
    ///
    /// A session token is typically obtained after a user successfully logs in
    /// and is used to authenticate subsequent requests for that user.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use parse_rs::{Parse, ParseError};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), ParseError> {
    /// # let mut client = Parse::new("http://localhost:1338/parse", "myAppId", None, None, Some("myMasterKey")).await?;
    /// // After a user logs in, the client might have a session token.
    /// if let Some(token) = client.session_token() {
    ///     println!("Current session token: {}", token);
    /// } else {
    ///     println!("No active session token.");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn session_token(&self) -> Option<&str> {
        self.session_token.as_deref()
    }

    /// Checks if the client currently has an active session token.
    ///
    /// This is a convenience method equivalent to `client.session_token().is_some()`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use parse_rs::{Parse, ParseError};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), ParseError> {
    /// # let mut client = Parse::new("http://localhost:1338/parse", "myAppId", None, None, Some("myMasterKey")).await?;
    /// if client.is_authenticated() {
    ///     println!("Client has an active session.");
    /// } else {
    ///     println!("Client does not have an active session.");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_authenticated(&self) -> bool {
        self.session_token.is_some()
    }

    /// Uploads a file to the Parse Server.
    ///
    /// This method sends the raw byte data of a file to the Parse Server, which then stores it
    /// and returns a `FileField` containing the URL and name of the stored file. This `FileField`
    /// can then be associated with a `ParseObject`.
    ///
    /// Note: File uploads require the Master Key to be configured on the `Parse` or for the
    /// `use_master_key` parameter in the underlying `_request_file_upload` to be true (which is the default for this public method).
    ///
    /// # Arguments
    ///
    /// * `file_name`: A string slice representing the desired name for the file on the server (e.g., `"photo.jpg"`).
    /// * `data`: A `Vec<u8>` containing the raw byte data of the file.
    /// * `mime_type`: A string slice representing the MIME type of the file (e.g., `"image/jpeg"`, `"application/pdf"`).
    ///
    /// # Returns
    ///
    /// A `Result` containing a `FileField` on success, which includes the `name` and `url` of the uploaded file.
    /// Returns a `ParseError` if the upload fails due to network issues, server errors, incorrect permissions,
    /// or misconfiguration.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use parse_rs::{Parse, ParseError, FileField, ParseObject, types::Value};
    /// use std::collections::HashMap;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), ParseError> {
    /// # let server_url = std::env::var("PARSE_SERVER_URL").unwrap_or_else(|_| "http://localhost:1338/parse".to_string());
    /// # let app_id = std::env::var("PARSE_APP_ID").unwrap_or_else(|_| "myAppId".to_string());
    /// # let master_key = std::env::var("PARSE_MASTER_KEY").unwrap_or_else(|_| "myMasterKey".to_string());
    /// # let mut client = Parse::new(&server_url, &app_id, None, None, Some(&master_key)).await?;
    ///
    /// let file_name = "profile.png";
    /// let file_data: Vec<u8> = vec![0, 1, 2, 3, 4, 5]; // Example byte data
    /// let mime_type = "image/png";
    ///
    /// // Upload the file
    /// let file_field: FileField = client.upload_file(file_name, file_data, mime_type).await?;
    ///
    /// println!("File uploaded successfully: Name - {}, URL - {}", file_field.name(), file_field.url());
    ///
    /// // Now, you can associate this FileField with a ParseObject
    /// let mut player_profile_data = HashMap::new();
    /// player_profile_data.insert("playerName".to_string(), Value::String("John Doe".to_string()));
    /// player_profile_data.insert("profilePicture".to_string(), Value::File(file_field));
    ///
    /// let mut player_profile = ParseObject::new("PlayerProfile", player_profile_data);
    /// let created_profile: ParseObject = client.create(&mut player_profile).await?;
    ///
    /// println!("Created PlayerProfile with ID: {}", created_profile.get_object_id().unwrap_or_default());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn upload_file(
        &self,
        file_name: &str,
        data: Vec<u8>,
        mime_type: &str,
    ) -> Result<FileField, ParseError> {
        let file_path_segment = format!("files/{}", file_name); // Path relative to /parse endpoint
        let server_url_str = self.server_url.as_str();

        let mut full_url_str: String;
        if server_url_str.ends_with("/parse") || server_url_str.ends_with("/parse/") {
            // server_url already contains /parse, e.g., http://domain/parse
            full_url_str = server_url_str.trim_end_matches('/').to_string();
            full_url_str = format!(
                "{}/{}",
                full_url_str,
                file_path_segment.trim_start_matches('/')
            );
        } else {
            // server_url is base, e.g., http://domain, needs /parse segment added
            full_url_str = format!(
                "{}/parse/{}",
                server_url_str.trim_end_matches('/'),
                file_path_segment.trim_start_matches('/')
            );
        }

        let final_url = Url::parse(&full_url_str)?;

        let mut request_builder = self.http_client.post(final_url.clone());

        // Set headers for file upload
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Parse-Application-Id",
            HeaderValue::from_str(&self.app_id).map_err(ParseError::InvalidHeaderValue)?,
        );
        // Master key is typically required for creating files directly, unless CLPs are very open.
        // If session token is present, it might be used depending on server config / CLPs.
        // For simplicity here, let's assume master key if no session token, or make it explicit if needed.
        if let Some(token) = &self.session_token {
            headers.insert(
                "X-Parse-Session-Token",
                HeaderValue::from_str(token).map_err(ParseError::InvalidHeaderValue)?,
            );
        } else if let Some(mk) = &self.master_key {
            headers.insert(
                "X-Parse-Master-Key",
                HeaderValue::from_str(mk).map_err(ParseError::InvalidHeaderValue)?,
            );
        } else {
            // Or return an error if neither is present and master key is strictly required
            // For now, proceed, relying on server's default behavior / public CLPs if any.
            log::warn!(
                "Uploading file without explicit master key or session token. Relies on CLPs."
            );
        }
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str(mime_type).map_err(ParseError::InvalidHeaderValue)?,
        );

        request_builder = request_builder.headers(headers);

        let data_len = data.len(); // Capture length before move
        request_builder = request_builder.body(data); // data is moved here

        // Log details before sending (similar to _request)
        log::debug!("--- Parse: Uploading File ---");
        log::debug!("URL: {}", final_url.as_str());
        log::debug!("Method: POST");
        // Headers are already part of request_builder, logging them directly from it is complex.
        // For now, we'll skip detailed header logging here, assuming _request's logging is the primary source.
        log::debug!("Content-Type: {}", mime_type);
        log::debug!("Body: <binary data of size {}>", data_len); // Use captured length
        log::debug!("-----------------------------------");

        let response = request_builder
            .send()
            .await
            .map_err(ParseError::ReqwestError)?;

        let upload_response: FileUploadResponse = self
            ._send_and_process_response(response, &file_path_segment)
            .await?; // Pass response and endpoint context

        Ok(FileField {
            _type: "File".to_string(),
            name: upload_response.name,
            url: upload_response.url,
        })
    }

    // Aggregate queries
    /// Executes an aggregation pipeline against a specified class and returns the results.
    ///
    /// Aggregation queries allow for complex data processing and computation directly on the server.
    /// The pipeline is defined as a `serde_json::Value`, typically an array of stages (e.g., `$match`, `$group`, `$sort`).
    /// This method requires the Master Key to be configured on the `Parse` and is used for the request.
    ///
    /// Refer to the [Parse Server aggregation documentation](https://docs.parseplatform.org/rest/guide/#aggregate) and
    /// [MongoDB aggregation pipeline documentation](https://www.mongodb.com/docs/manual/core/aggregation-pipeline/) for details on constructing pipelines.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The type that each element of the result set is expected to deserialize into. This type must implement `DeserializeOwned`.
    ///
    /// # Arguments
    ///
    /// * `class_name`: The name of the class to perform the aggregation on (e.g., `"GameScore"`).
    /// * `pipeline`: A `serde_json::Value` representing the aggregation pipeline. This is usually a JSON array.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Vec<T>` where `T` is the deserialized type of the aggregation results.
    /// Returns a `ParseError` if the aggregation fails, the pipeline is invalid, or the Master Key is not available.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use parse_rs::{Parse, ParseError};
    /// use serde::Deserialize;
    /// use serde_json::json; // for constructing the pipeline value
    ///
    /// #[derive(Deserialize, Debug)]
    /// struct PlayerStats {
    ///     // Note: Parse Server might return grouped _id as "objectId"
    ///     #[serde(rename = "objectId")]
    ///     player_name: String,
    ///     total_score: i64,
    ///     average_score: f64,
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), ParseError> {
    /// # let server_url = std::env::var("PARSE_SERVER_URL").unwrap_or_else(|_| "http://localhost:1338/parse".to_string());
    /// # let app_id = std::env::var("PARSE_APP_ID").unwrap_or_else(|_| "myAppId".to_string());
    /// # let master_key = std::env::var("PARSE_MASTER_KEY").unwrap_or_else(|_| "myMasterKey".to_string());
    /// let client = Parse::new(&server_url, &app_id, None, None, Some(&master_key)).await?;
    /// let class_name = "GameScore";
    /// let pipeline = json!([
    ///     { "$match": { "playerName": { "$exists": true } } },
    ///     { "$group": {
    ///         "_id": "$playerName",
    ///         "totalScore": { "$sum": "$score" },
    ///         "averageScore": { "$avg": "$score" }
    ///     }},
    ///     { "$sort": { "totalScore": -1 } },
    ///     { "$project": {
    ///         "_id": 0, // Exclude the default _id field from MongoDB if not needed
    ///         "playerName": "$_id", // Rename _id to playerName
    ///         "total_score": "$totalScore",
    ///         "average_score": "$averageScore"
    ///     }}
    /// ]);
    ///
    /// let results: Vec<PlayerStats> = client.execute_aggregate(class_name, pipeline).await?;
    ///
    /// for stats in results {
    ///     println!("Player: {}, Total Score: {}, Avg Score: {:.2}",
    ///              stats.player_name, stats.total_score, stats.average_score);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute_aggregate<T: DeserializeOwned + Send + 'static>(
        &self,
        class_name: &str,
        pipeline: Value, // Assuming pipeline is a serde_json::Value (e.g., array of stages)
    ) -> Result<Vec<T>, ParseError> {
        let endpoint = format!("aggregate/{}", class_name);
        // Serialize the pipeline to a JSON string
        let pipeline_str = serde_json::to_string(&pipeline).map_err(|e| {
            ParseError::SerializationError(format!("Failed to serialize pipeline: {}", e))
        })?;

        // Construct query parameters
        let params = vec![("pipeline".to_string(), pipeline_str)];

        // Deserialize into AggregateResponse<T> first
        let response_wrapper: AggregateResponse<T> = self
            ._get_with_url_params(&endpoint, &params, true, None)
            .await?;

        Ok(response_wrapper.results) // Then extract the results vector
    }

    /// Deletes an object from a class using the Master Key.
    ///
    /// This method provides a direct way to delete any object by its class name and object ID,
    /// bypassing ACLs and Class-Level Permissions due to the use of the Master Key.
    /// The Master Key must be configured on the `Parse` for this operation to succeed.
    ///
    /// For more general object deletion that respects ACLs and uses the current session's
    /// authentication, use the `delete` method on a `ParseObject` instance retrieved via the client,
    /// or the `delete` method available on the `ParseUserHandle` for users.
    ///
    /// # Arguments
    ///
    /// * `endpoint`: A string slice representing the relative path to the object, typically in the
    ///   format `"classes/ClassName/objectId"` (e.g., `"classes/GameScore/xWMyZ4YEGZ"`).
    ///
    /// # Returns
    ///
    /// A `Result` containing a `serde_json::Value` (which is usually an empty JSON object `{}`
    /// upon successful deletion by the Parse Server) or a `ParseError` if the deletion fails
    /// (e.g., object not found, Master Key not configured, network issue).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use parse_rs::{Parse, ParseError};
    /// use serde_json::Value;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), ParseError> {
    /// # let server_url = std::env::var("PARSE_SERVER_URL").unwrap_or_else(|_| "http://localhost:1338/parse".to_string());
    /// # let app_id = std::env::var("PARSE_APP_ID").unwrap_or_else(|_| "myAppId".to_string());
    /// # let master_key = std::env::var("PARSE_MASTER_KEY").unwrap_or_else(|_| "myMasterKey".to_string());
    /// # let client = Parse::new(&server_url, &app_id, None, None, Some(&master_key)).await?;
    ///
    /// let class_name = "OldGameData";
    /// let object_id_to_delete = "someObjectId123";
    /// let endpoint_to_delete = format!("classes/{}/{}", class_name, object_id_to_delete);
    ///
    /// // Ensure client is configured with Master Key for this to work
    /// match client.delete_object_with_master_key(&endpoint_to_delete).await {
    ///     Ok(_) => println!("Successfully deleted object {} from class {}.", object_id_to_delete, class_name),
    ///     Err(e) => eprintln!("Failed to delete object: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_object_with_master_key(
        &self,
        endpoint: &str, // Expects relative endpoint like "classes/MyClass/objectId"
    ) -> Result<Value, ParseError> {
        if self.master_key.is_none() {
            return Err(ParseError::MasterKeyRequired(
                "Master key is required for delete_with_master_key but not configured.".to_string(),
            ));
        }
        self._request(Method::DELETE, endpoint, None::<&Value>, true, None) // Pass relative endpoint
            .await
    }

    /// Executes a `ParseQuery` and returns a list of matching objects.
    ///
    /// # Arguments
    /// * `query`: A reference to the `ParseQuery` to execute.
    ///
    /// # Returns
    /// A `Result` containing a `Vec<T>` of the deserialized objects or a `ParseError`.
    pub async fn execute_query<T: DeserializeOwned + Send + Sync + 'static>(
        &self,
        query: &crate::query::ParseQuery, // Corrected: ParseQuery is not generic itself
    ) -> Result<Vec<T>, ParseError> {
        let class_name = query.class_name();
        let base_endpoint = format!("classes/{}", class_name);
        let params = query.build_query_params(); // Assuming this method exists on ParseQuery

        // Queries generally do not require the master key by default.
        // Auth is typically handled by session token or API keys based on ACLs.
        let use_master_key = query.uses_master_key(); // Check if query explicitly needs master key
        let session_token_to_use = self.session_token.as_deref();

        let response: QueryResponse<T> = self
            ._get_with_url_params(
                &base_endpoint, // Pass relative endpoint
                &params,
                use_master_key,
                session_token_to_use,
            )
            .await?;
        Ok(response.results)
    }

    /// Executes a `ParseQuery` and returns a list of `ParseObject` instances,
    /// ensuring the `class_name` field is populated for each object from the query.
    ///
    /// # Arguments
    /// * `query`: A reference to the `ParseQuery` to execute.
    ///
    /// # Returns
    /// A `Result` containing a `Vec<ParseObject>` or a `ParseError`.
    pub async fn find_objects(
        &self,
        query: &crate::query::ParseQuery,
    ) -> Result<Vec<ParseObject>, ParseError> {
        let mut objects: Vec<ParseObject> = self.execute_query(query).await?;
        let class_name_from_query = query.class_name().to_string();

        for object in objects.iter_mut() {
            object.class_name = class_name_from_query.clone();
        }

        Ok(objects)
    }

    /// Creates a new class schema in your Parse application.
    ///
    /// This operation requires the Master Key to be configured on the `Parse`
    /// and will use it for authentication.
    ///
    /// # Arguments
    ///
    /// * `class_name`: The name of the class to create. This must match the `className` field in the `schema_payload`.
    /// * `schema_payload`: A `serde_json::Value` representing the schema to create. It must include
    ///   `className` and `fields`. Optionally, `classLevelPermissions` and `indexes` can be included.
    ///   Example:
    ///   ```json
    ///   {
    ///     "className": "MyNewClass",
    ///     "fields": {
    ///       "name": { "type": "String", "required": true },
    ///       "score": { "type": "Number", "defaultValue": 0 }
    ///     },
    ///     "classLevelPermissions": {
    ///       "find": { "*": true },
    ///       "get": { "*": true }
    ///     }
    ///   }
    ///   ```
    ///
    /// # Returns
    ///
    /// A `Result` containing the `ParseSchema` of the newly created class,
    /// or a `ParseError` if the request fails (e.g., Master Key not provided, schema definition error, class already exists, network error).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use parse_rs::Parse;
    /// use serde_json::json;
    /// # use parse_rs::ParseError;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), ParseError> {
    /// # let server_url = std::env::var("PARSE_SERVER_URL").unwrap_or_else(|_| "http://localhost:1338/parse".to_string());
    /// # let app_id = std::env::var("PARSE_APP_ID").unwrap_or_else(|_| "myAppId".to_string());
    /// # let master_key = std::env::var("PARSE_MASTER_KEY").unwrap_or_else(|_| "myMasterKey".to_string());
    /// let client = Parse::new(&server_url, &app_id, None, None, Some(&master_key)).await?;
    /// let new_class_name = "MyTemporaryClass";
    ///
    /// let schema_payload = json!({
    ///     "className": new_class_name,
    ///     "fields": {
    ///         "playerName": { "type": "String" },
    ///         "score": { "type": "Number", "required": true, "defaultValue": 0 }
    ///     },
    ///     "classLevelPermissions": {
    ///         "find": { "*": true },
    ///         "get": { "*": true },
    ///         "create": { "*": true }, // Allow creation for testing
    ///         "update": { "*": true }, // Allow update for testing
    ///         "delete": { "*": true }  // Allow deletion for testing
    ///     }
    /// });
    ///
    /// match client.create_class_schema(new_class_name, &schema_payload).await {
    ///     Ok(schema) => {
    ///         println!("Successfully created schema for class '{}':", schema.class_name);
    ///         println!("Fields: {:?}", schema.fields.keys());
    ///         // You can now create objects of this class, e.g., using client.create_object(...)
    ///     }
    ///     Err(e) => eprintln!("Failed to create schema for class '{}': {}", new_class_name, e),
    /// }
    ///
    /// // Clean up: Delete the class schema (optional, for testing)
    /// // Ensure the class is empty before deleting, or set drop_class_if_objects_exist to true.
    /// client.delete_class_schema(new_class_name, true).await.ok();
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_class_schema<T: Serialize + Send + Sync>(
        &self,
        class_name: &str, // class_name in path must match className in body
        schema_payload: &T,
    ) -> Result<ParseSchema, ParseError> {
        if self.master_key.is_none() {
            return Err(ParseError::MasterKeyRequired(format!(
                "Master key is required to create schema for class '{}'.",
                class_name
            )));
        }

        let endpoint = format!("schemas/{}", class_name);
        self._request(
            Method::POST,
            &endpoint,
            Some(schema_payload),
            true, // Use master key
            None, // No session token override
        )
        .await
    }

    /// Updates the schema for an existing class in your Parse application.
    ///
    /// This can be used to add or remove fields, change field types (with caution),
    /// update Class-Level Permissions (CLP), or add/remove indexes.
    /// To delete a field or index, use the `{"__op": "Delete"}` operator in the payload.
    ///
    /// This operation requires the Master Key to be configured on the `Parse`
    /// and will use it for authentication.
    ///
    /// # Arguments
    ///
    /// * `class_name`: The name of the class whose schema is to be updated.
    /// * `schema_update_payload`: A `serde_json::Value` representing the changes to apply.
    ///   Example to add a field and delete another:
    ///   ```json
    ///   {
    ///     "className": "MyExistingClass", // Should match class_name argument
    ///     "fields": {
    ///       "newField": { "type": "Boolean" },
    ///       "oldField": { "__op": "Delete" }
    ///     },
    ///     "classLevelPermissions": {
    ///       "update": { "role:Admin": true } // Example CLP update
    ///     }
    ///   }
    ///   ```
    ///
    /// # Returns
    ///
    /// A `Result` containing the updated `ParseSchema` of the class,
    /// or a `ParseError` if the request fails (e.g., Master Key not provided, class not found, invalid update operation, network error).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use parse_rs::Parse;
    /// use serde_json::json;
    /// # use parse_rs::ParseError;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), ParseError> {
    /// # let server_url = std::env::var("PARSE_SERVER_URL").unwrap_or_else(|_| "http://localhost:1338/parse".to_string());
    /// # let app_id = std::env::var("PARSE_APP_ID").unwrap_or_else(|_| "myAppId".to_string());
    /// # let master_key = std::env::var("PARSE_MASTER_KEY").unwrap_or_else(|_| "myMasterKey".to_string());
    /// let client = Parse::new(&server_url, &app_id, None, None, Some(&master_key)).await?;
    /// let class_to_update = "MyUpdatableClass";
    ///
    /// // 1. Ensure the class exists (create it for the example)
    /// let initial_payload = json!({
    ///    "className": class_to_update,
    ///    "fields": { "initialField": { "type": "String" } },
    ///    "classLevelPermissions": { "find": {"*": true}, "get": {"*": true}, "create": {"*": true}, "update": {"*": true}, "delete": {"*": true} }
    /// });
    /// client.create_class_schema(class_to_update, &initial_payload).await.ok(); // Ignore error if already exists
    ///
    /// // 2. Prepare the update payload
    /// let update_payload = json!({
    ///     "className": class_to_update, // Must match
    ///     "fields": {
    ///         "addedField": { "type": "Number" },
    ///         "initialField": { "__op": "Delete" } // Delete the initial field
    ///     },
    ///     "classLevelPermissions": {
    ///         "get": { "role:Moderator": true, "*": false } // Change CLP
    ///     }
    /// });
    ///
    /// match client.update_class_schema(class_to_update, &update_payload).await {
    ///     Ok(schema) => {
    ///         println!("Successfully updated schema for class '{}':", schema.class_name);
    ///         println!("Current Fields: {:?}", schema.fields.keys());
    ///         if let Some(clp) = schema.class_level_permissions {
    ///             println!("Current CLP (get): {:?}", clp.get);
    ///         }
    ///     }
    ///     Err(e) => eprintln!("Failed to update schema for class '{}': {}", class_to_update, e),
    /// }
    ///
    /// // Clean up: Delete the class schema (optional, for testing)
    /// client.delete_class_schema(class_to_update, true).await.ok();
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_class_schema<T: Serialize + Send + Sync>(
        &self,
        class_name: &str,
        schema_update_payload: &T,
    ) -> Result<ParseSchema, ParseError> {
        if self.master_key.is_none() {
            return Err(ParseError::MasterKeyRequired(format!(
                "Master key is required to update schema for class '{}'.",
                class_name
            )));
        }

        let endpoint = format!("schemas/{}", class_name);
        self._request(
            Method::PUT,
            &endpoint,
            Some(schema_update_payload),
            true, // Use master key
            None, // No session token override
        )
        .await
    }

    /// Fetches the schema for a specific class in your Parse application.
    ///
    /// This operation requires the Master Key to be configured on the `Parse`
    /// and will use it for authentication.
    ///
    /// # Arguments
    ///
    /// * `class_name`: The name of the class for which to fetch the schema.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `ParseSchema` for the specified class,
    /// or a `ParseError` if the request fails (e.g., Master Key not provided, class not found, network error).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use parse_rs::Parse;
    /// # use parse_rs::ParseError;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), ParseError> {
    /// # let server_url = std::env::var("PARSE_SERVER_URL").unwrap_or_else(|_| "http://localhost:1338/parse".to_string());
    /// # let app_id = std::env::var("PARSE_APP_ID").unwrap_or_else(|_| "myAppId".to_string());
    /// # let master_key = std::env::var("PARSE_MASTER_KEY").unwrap_or_else(|_| "myMasterKey".to_string());
    /// let client = Parse::new(&server_url, &app_id, None, None, Some(&master_key)).await?;
    /// let class_to_fetch = "MyTestClass"; // Assume this class exists
    ///
    /// // First, ensure the class exists by creating it if it doesn't (for testability)
    /// // For a real scenario, you'd likely expect the class to exist.
    /// let initial_schema_payload = serde_json::json!({
    ///    "className": class_to_fetch,
    ///    "fields": {
    ///        "someField": { "type": "String" }
    ///    }
    /// });
    /// client.create_class_schema(class_to_fetch, &initial_schema_payload).await.ok(); // Ignore error if already exists
    ///
    /// match client.get_class_schema(class_to_fetch).await {
    ///     Ok(schema) => {
    ///         println!("Successfully fetched schema for class '{}':", schema.class_name);
    ///         println!("Fields: {:?}", schema.fields.keys());
    ///         if let Some(clp) = &schema.class_level_permissions {
    ///             println!("CLP: {:?}", clp.get);
    ///         }
    ///     }
    ///     Err(e) => eprintln!("Failed to fetch schema for class '{}': {}", class_to_fetch, e),
    /// }
    ///
    /// // Clean up the test class (optional)
    /// client.delete_class_schema(class_to_fetch, true).await.ok();
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_class_schema(&self, class_name: &str) -> Result<ParseSchema, ParseError> {
        if self.master_key.is_none() {
            return Err(ParseError::MasterKeyRequired(format!(
                "Master key is required to fetch schema for class '{}'.",
                class_name
            )));
        }

        let endpoint = format!("schemas/{}", class_name);
        self._request(
            Method::GET,
            &endpoint,
            None::<&Value>, // No body for GET request
            true,           // Use master key
            None,           // No session token override
        )
        .await
    }

    /// Deletes an existing class schema from your Parse application.
    ///
    /// **Important:** The class must be empty (contain no objects) for the deletion to succeed.
    /// If the class contains objects, the Parse Server will return an error.
    ///
    /// This operation requires the Master Key to be configured on the `Parse`
    /// and will use it for authentication.
    ///
    /// # Arguments
    ///
    /// * `class_name`: The name of the class whose schema is to be deleted.
    /// * `fail_if_objects_exist`: If `true` (the default), the operation will fail if the class contains objects.
    ///   Currently, the Parse API does not support automatically deleting objects along with the schema in a single call.
    ///   You must delete all objects from the class manually before calling this method if you want to delete a non-empty class.
    ///   Setting this to `false` is not currently supported by the underlying API and will behave like `true`.
    ///
    /// # Returns
    ///
    /// A `Result<(), ParseError>` which is `Ok(())` on successful deletion,
    /// or a `ParseError` if the request fails (e.g., Master Key not provided, class not found, class not empty, network error).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use parse_rs::Parse;
    /// use serde_json::json;
    /// # use parse_rs::ParseError;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), ParseError> {
    /// # let server_url = std::env::var("PARSE_SERVER_URL").unwrap_or_else(|_| "http://localhost:1338/parse".to_string());
    /// # let app_id = std::env::var("PARSE_APP_ID").unwrap_or_else(|_| "myAppId".to_string());
    /// # let master_key = std::env::var("PARSE_MASTER_KEY").unwrap_or_else(|_| "myMasterKey".to_string());
    /// let client = Parse::new(&server_url, &app_id, None, None, Some(&master_key)).await?;
    /// let class_to_delete = "MyClassToDelete";
    ///
    /// // 1. Create a class for the example (ensure it's empty for successful deletion)
    /// let schema_payload = json!({
    ///     "className": class_to_delete,
    ///     "fields": { "tempField": { "type": "String" } },
    ///     "classLevelPermissions": { "find": {"*": true}, "get": {"*": true}, "create": {"*": true}, "update": {"*": true}, "delete": {"*": true} }
    /// });
    /// client.create_class_schema(class_to_delete, &schema_payload).await.ok(); // Create it, ignore if already exists for test idempotency
    ///
    /// // 2. Attempt to delete the (empty) class schema
    /// match client.delete_class_schema(class_to_delete, true).await {
    ///     Ok(()) => println!("Successfully deleted schema for class '{}'", class_to_delete),
    ///     Err(e) => eprintln!("Failed to delete schema for class '{}': {}. Ensure it's empty.", class_to_delete, e),
    /// }
    ///
    /// // Example of trying to delete a class that might not be empty (will likely fail if objects exist)
    /// // let another_class = "PotentiallyNonEmptyClass";
    /// // if let Err(e) = client.delete_class_schema(another_class, true).await {
    /// //     eprintln!("Could not delete schema for '{}': {}. It might not be empty or might not exist.", another_class, e);
    /// // }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_class_schema(
        &self,
        class_name: &str,
        _fail_if_objects_exist: bool, // Parameter kept for future API changes, currently server enforces emptiness
    ) -> Result<(), ParseError> {
        if self.master_key.is_none() {
            return Err(ParseError::MasterKeyRequired(format!(
                "Master key is required to delete schema for class '{}'.",
                class_name
            )));
        }

        let endpoint = format!("schemas/{}", class_name);
        // The response for a successful DELETE is often an empty JSON object {} or no content.
        // We map it to Ok(()) if successful.
        let _response: Value = self
            ._request(
                Method::DELETE,
                &endpoint,
                None::<&Value>, // No body for DELETE request
                true,           // Use master key
                None,           // No session token override
            )
            .await?;
        Ok(())
    }

    /// Methods to get handles for specific Parse features
    /// Returns a `ParseUserHandle` for managing user authentication and user-specific operations.
    ///
    /// The `ParseUserHandle` provides methods like `signup`, `login`, `logout`, `request_password_reset`,
    /// `get_current_user`, `update_current_user`, and `delete_current_user`.
    /// It operates in the context of the current `Parse` instance, using its configuration
    /// and session state.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use parse_rs::{Parse, ParseError, types::Value};
    /// use std::collections::HashMap;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), ParseError> {
    /// # let server_url = std::env::var("PARSE_SERVER_URL").unwrap_or_else(|_| "http://localhost:1338/parse".to_string());
    /// # let app_id = std::env::var("PARSE_APP_ID").unwrap_or_else(|_| "myAppId".to_string());
    /// # let mut client = Parse::new(&server_url, &app_id, None, None, None).await?;
    ///
    /// let mut user_data = HashMap::new();
    /// user_data.insert("email".to_string(), Value::String("test@example.com".to_string()));
    /// // Add other fields as needed for signup
    ///
    /// // Get the user handle and sign up a new user
    /// // let new_user = client.user().signup("testuser", "password123", Some(user_data)).await?;
    /// // println!("New user signed up with ID: {}", new_user.get_object_id().unwrap_or_default());
    ///
    /// // Later, to log in:
    /// // let logged_in_user = client.user().login("testuser", "password123").await?;
    /// // println!("User logged in. Session token: {}", client.session_token().unwrap_or_default());
    /// # Ok(())
    /// # }
    /// ```
    pub fn user(&mut self) -> ParseUserHandle<'_> {
        ParseUserHandle::new(self)
    }

    /// Returns a `ParseSessionHandle` for managing session-specific operations.
    ///
    /// The `ParseSessionHandle` provides methods like `get_current_session` (to validate the current client's session token)
    /// and `delete_session` (to delete a specific session, requires Master Key).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use parse_rs::{Parse, ParseError, ParseSession};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), ParseError> {
    /// # let server_url = std::env::var("PARSE_SERVER_URL").unwrap_or_else(|_| "http://localhost:1338/parse".to_string());
    /// # let app_id = std::env::var("PARSE_APP_ID").unwrap_or_else(|_| "myAppId".to_string());
    /// # let mut client = Parse::new(&server_url, &app_id, None, None, None).await?;
    ///
    /// // After a user logs in, their session token is stored in the client.
    /// // You can then get details about the current session:
    /// // if client.is_authenticated() {
    /// //     match client.session().get_current_session().await {
    /// //         Ok(current_session_details) => {
    /// //             println!("Current session is valid for user: {}",
    /// //                      current_session_details.get_user().map_or("N/A", |u| u.get_object_id().unwrap_or_default()));
    /// //         }
    /// //         Err(e) => eprintln!("Could not get current session details: {}", e),
    /// //     }
    /// // }
    /// # Ok(())
    /// # }
    /// ```
    pub fn session(&self) -> crate::session::ParseSessionHandle<'_> {
        crate::session::ParseSessionHandle::new(self)
    }

    /// Returns a `ParseCloud` handle for calling Parse Cloud Code functions.
    ///
    /// The `ParseCloud` handle provides the `call_function` method to execute server-side Cloud Code.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use parse_rs::{Parse, ParseError};
    /// use serde_json::json; // For creating parameters
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), ParseError> {
    /// # let server_url = std::env::var("PARSE_SERVER_URL").unwrap_or_else(|_| "http://localhost:1338/parse".to_string());
    /// # let app_id = std::env::var("PARSE_APP_ID").unwrap_or_else(|_| "myAppId".to_string());
    /// # let client = Parse::new(&server_url, &app_id, None, None, None).await?;
    ///
    /// let function_name = "helloWorld";
    /// let params = json!({ "name": "Rustaceans" });
    ///
    /// // match client.cloud().call_function(function_name, Some(params)).await {
    /// //     Ok(result) => println!("Cloud function '{}' returned: {}", function_name, result),
    /// //     Err(e) => eprintln!("Cloud function '{}' failed: {}", function_name, e),
    /// // }
    /// # Ok(())
    /// # }
    /// ```
    pub fn cloud(&self) -> ParseCloud<'_> {
        ParseCloud::new(self)
    }

    /// Fetches the schemas for all classes in your Parse application.
    ///
    /// This operation requires the Master Key to be configured on the `Parse`
    /// and will use it for authentication.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `GetAllSchemasResponse` which includes a list of `ParseSchema` objects,
    /// or a `ParseError` if the request fails (e.g., Master Key not provided, network error).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use parse_rs::Parse;
    /// # use parse_rs::ParseError;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), ParseError> {
    /// # let server_url = std::env::var("PARSE_SERVER_URL").unwrap_or_else(|_| "http://localhost:1338/parse".to_string());
    /// # let app_id = std::env::var("PARSE_APP_ID").unwrap_or_else(|_| "myAppId".to_string());
    /// # let master_key = std::env::var("PARSE_MASTER_KEY").unwrap_or_else(|_| "myMasterKey".to_string());
    /// let client = Parse::new(&server_url, &app_id, None, None, Some(&master_key)).await?;
    ///
    /// match client.get_all_schemas().await {
    ///     Ok(response) => {
    ///         println!("Successfully fetched {} schemas:", response.results.len());
    ///         for schema in response.results {
    ///             println!("- Class: {}, Fields: {:?}", schema.class_name, schema.fields.keys());
    ///         }
    ///     }
    ///     Err(e) => eprintln!("Failed to fetch schemas: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_all_schemas(&self) -> Result<GetAllSchemasResponse, ParseError> {
        if self.master_key.is_none() {
            return Err(ParseError::MasterKeyRequired(
                "Master key is required to fetch all schemas.".to_string(),
            ));
        }

        self._request(
            Method::GET,
            "schemas",
            None::<&Value>, // No body for GET request
            true,           // Use master key
            None,           // No session token override
        )
        .await
    }
}

// Temporary struct for deserializing file upload response
#[derive(serde::Deserialize, Debug)]
struct FileUploadResponse {
    name: String,
    url: String,
}

/// Response for a successful config update.
#[derive(serde::Deserialize, Debug)]
pub struct UpdateConfigResponse {
    pub result: bool,
}

// Response for aggregate queries
#[derive(serde::Deserialize, Debug)]
struct AggregateResponse<T> {
    results: Vec<T>,
}

// Response for standard queries
#[derive(serde::Deserialize, Debug)]
pub struct QueryResponse<T> {
    // Made public for potential use in ParseQuery if it ever handles responses directly
    pub results: Vec<T>,
}

// Helper method for GET requests with URL parameters (e.g., queries, aggregations)
impl Parse {
    pub(crate) async fn _get_with_url_params<R: DeserializeOwned + Send + 'static>(
        &self,
        endpoint: &str,
        params: &[(String, String)],
        use_master_key: bool,
        session_token_override: Option<&str>,
    ) -> Result<R, ParseError> {
        let base_url = Url::parse(&self.server_url).map_err(|e| {
            ParseError::InvalidUrl(format!(
                "Base server URL '{}' is invalid: {}",
                self.server_url, e
            ))
        })?;

        let api_path = format!("/parse/{}", endpoint.trim_start_matches('/'));

        let mut full_url = base_url.join(&api_path).map_err(|e| {
            ParseError::InvalidUrl(format!(
                "Failed to join base URL '{}' with API path '{}': {}",
                base_url, api_path, e
            ))
        })?;

        // Add query parameters
        if !params.is_empty() {
            for (key, value) in params {
                full_url.query_pairs_mut().append_pair(key, value);
            }
        }

        log::debug!(
            "Preparing GET request with params: URL={}, UseMasterKey={}, SessionTokenOverride={:?}",
            full_url.as_str(),
            use_master_key,
            session_token_override
        );

        let mut request_builder = self.http_client.get(full_url.clone());

        // Apply authentication headers based on context
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Parse-Application-Id",
            HeaderValue::from_str(&self.app_id).map_err(ParseError::InvalidHeaderValue)?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        // Authentication headers - applied in order of precedence
        if let Some(token_override) = session_token_override {
            headers.insert(
                "X-Parse-Session-Token",
                HeaderValue::from_str(token_override).map_err(ParseError::InvalidHeaderValue)?,
            );
        } else if use_master_key {
            if let Some(master_key) = &self.master_key {
                headers.insert(
                    "X-Parse-Master-Key",
                    HeaderValue::from_str(master_key).map_err(ParseError::InvalidHeaderValue)?,
                );
            } else {
                log::warn!("Master key requested for operation but not configured.");
                return Err(ParseError::MasterKeyRequired(
                    "Master key is required for this operation but not configured.".to_string(),
                ));
            }
        } else if let Some(session_token) = &self.session_token {
            // Client's default session token
            headers.insert(
                "X-Parse-Session-Token",
                HeaderValue::from_str(session_token).map_err(ParseError::InvalidHeaderValue)?,
            );
        } else if let Some(js_key) = &self.javascript_key {
            headers.insert(
                "X-Parse-Javascript-Key",
                HeaderValue::from_str(js_key).map_err(ParseError::InvalidHeaderValue)?,
            );
        } else if let Some(rest_key) = &self.rest_api_key {
            // Fallback to REST API Key
            headers.insert(
                "X-Parse-REST-API-Key",
                HeaderValue::from_str(rest_key).map_err(ParseError::InvalidHeaderValue)?,
            );
        }

        request_builder = request_builder.headers(headers.clone()); // Clone headers for logging if needed

        // Log request details before sending
        if log::log_enabled!(log::Level::Debug) {
            log::debug!("--- Parse GET Request --- ");
            log::debug!("URL: {}", full_url.as_str());
            log::debug!("Method: GET");
            for (name, value) in headers.iter() {
                log::debug!(
                    "Header: {}: {:?}",
                    name.as_str(),
                    value.to_str().unwrap_or("[non-ASCII value]")
                );
            }
            log::debug!("------------------------------");
        }

        // Perform the actual HTTP request
        let response = request_builder
            .send()
            .await
            .map_err(ParseError::ReqwestError)?;

        // Log response status and headers (conditionally)
        if log::log_enabled!(log::Level::Debug) {
            log::debug!("--- Parse Response ---");
            log::debug!("Status: {}", response.status());
            for (name, value) in response.headers() {
                log::debug!("Header: {}: {:?}", name, value);
            }
        }

        let status = response.status();
        if status.is_success() {
            let body_bytes = response.bytes().await.map_err(ParseError::ReqwestError)?;
            log::debug!(
                "Request successful. Response body: {}",
                String::from_utf8_lossy(&body_bytes)
            );
            serde_json::from_slice(&body_bytes).map_err(|e| {
                ParseError::JsonDeserializationFailed(format!(
                    "Error: {}, Body: {}",
                    e,
                    String::from_utf8_lossy(&body_bytes).into_owned()
                ))
            })
        } else {
            let error_body_bytes = response.bytes().await.map_err(ParseError::ReqwestError)?;
            let error_body_string = String::from_utf8_lossy(&error_body_bytes).to_string();
            log::warn!(
                "Request failed with status {} and body: {}",
                status,
                error_body_string
            );
            match serde_json::from_slice::<Value>(&error_body_bytes) {
                Ok(json_value) => Err(ParseError::from_response(status.as_u16(), json_value)),
                Err(_) => {
                    let fallback_json = serde_json::json!({
                        "code": status.as_u16(),
                        "error": error_body_string
                    });
                    Err(ParseError::from_response(status.as_u16(), fallback_json))
                }
            }
        }
    }

    // Central request method
    pub(crate) async fn _request<
        T: Serialize + Send + Sync,
        R: DeserializeOwned + Send + 'static,
    >(
        &self,
        method: Method,
        endpoint: &str, // Takes relative endpoint string
        body: Option<&T>,
        use_master_key: bool,
        session_token_override: Option<&str>,
    ) -> Result<R, ParseError> {
        let base_url = Url::parse(&self.server_url).map_err(|e| {
            ParseError::InvalidUrl(format!(
                "Base server URL '{}' is invalid: {}",
                self.server_url, e
            ))
        })?;

        // Ensure the endpoint starts with "/parse/" and then the specific API path.
        // Trim any leading slashes from the original endpoint to avoid issues like "/parse//classes".
        let api_path = format!("/parse/{}", endpoint.trim_start_matches('/'));

        let full_url = base_url.join(&api_path).map_err(|e| {
            ParseError::InvalidUrl(format!(
                "Failed to join base URL '{}' with API path '{}': {}",
                base_url, api_path, e
            ))
        })?;

        log::debug!(
            "Preparing request: Method={}, URL={}, UseMasterKey={}, SessionTokenOverride={:?}",
            method,
            full_url.as_str(), // Log the full_url
            use_master_key,
            session_token_override
        );

        let mut request_builder = self.http_client.request(method.clone(), full_url.clone());

        let mut headers = HeaderMap::new(); // Start with an empty map for request-specific headers

        // Determine effective session token
        let effective_session_token = session_token_override.or(self.session_token.as_deref());

        if let Some(token) = effective_session_token {
            headers.insert(
                "X-Parse-Session-Token",
                HeaderValue::from_str(token).map_err(ParseError::InvalidHeaderValue)?,
            );
        } else if use_master_key {
            // Only add Master Key if no session token is being used for this request
            if let Some(master_key) = &self.master_key {
                headers.insert(
                    "X-Parse-Master-Key",
                    HeaderValue::from_str(master_key).map_err(ParseError::InvalidHeaderValue)?,
                );
            } else {
                log::warn!("Master key requested for operation but not configured for the client.");
                return Err(ParseError::MasterKeyRequired(
                    "Master key is required for this operation but not configured on the client."
                        .to_string(),
                ));
            }
        }
        // Note: App ID and User-Agent are part of http_client.default_headers().
        // If no session token or master key is specified here, and if the client was initialized
        // with a JS key or REST key in its default_headers, those will be used by reqwest.

        if method == Method::POST || method == Method::PUT || method == Method::PATCH {
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        }

        let mut body_str_for_log: Option<String> = None;
        if let Some(body_data) = body {
            let body_str =
                serde_json::to_string_pretty(body_data).map_err(ParseError::JsonError)?;
            body_str_for_log = Some(body_str.clone());
            request_builder = request_builder.body(body_str);
        }

        // Apply the request-specific headers. The http_client will merge these with its defaults.
        request_builder = request_builder.headers(headers.clone()); // Clone headers for logging if needed

        // For logging, we want to see the effective headers. Reqwest doesn't easily show
        // the final merged headers before sending. So, we'll log what we're adding,
        // acknowledging that http_client adds its defaults (AppID, UserAgent, potentially initial JS/REST/Master key).
        log::debug!(
            "Preparing request: Method={}, URL={}, UseMasterKey={}, SessionTokenOverride={:?}",
            method,
            full_url,
            use_master_key,
            session_token_override
        );

        if let Some(log_body) = &body_str_for_log {
            log::debug!("Request body: {}", log_body);
        } else {
            log::debug!("Request body: None");
        }

        // Send the request
        let response = request_builder
            .send()
            .await
            .map_err(ParseError::ReqwestError)?;

        // Process the response
        if response.status().is_success() {
            // For 204 No Content, deserialize to a default value if R is Option or unit type
            if response.status() == reqwest::StatusCode::NO_CONTENT {
                return serde_json::from_str("{}").map_err(ParseError::JsonError);
            }
            let body_bytes = response.bytes().await.map_err(ParseError::ReqwestError)?;
            log::debug!(
                "Request successful. Response body: {}",
                String::from_utf8_lossy(&body_bytes)
            );
            serde_json::from_slice(&body_bytes).map_err(ParseError::JsonError)
        } else {
            let status = response.status();
            let error_body_bytes = response.bytes().await.map_err(ParseError::ReqwestError)?;
            let error_body_str = String::from_utf8_lossy(&error_body_bytes).to_string();
            log::warn!(
                "Request failed with status {}. Response body: {}",
                status,
                error_body_str
            );
            match serde_json::from_slice::<Value>(&error_body_bytes) {
                Ok(json_value) => Err(ParseError::from_response(status.as_u16(), json_value)),
                Err(_) => {
                    let fallback_json = serde_json::json!({
                        "code": status.as_u16(),
                        "error": error_body_str
                    });
                    Err(ParseError::from_response(status.as_u16(), fallback_json))
                }
            }
        }
    }
}
