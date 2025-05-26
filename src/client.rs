// src/client.rs

use crate::error::ParseError;
use crate::user::ParseUserHandle;
use crate::FileField;
use crate::ParseCloud;
use crate::ParseObject;

use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use reqwest::{Client, Method, Url};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

/// Enum to specify the type of authentication to use for a request.
pub enum AuthType {
    SessionToken,
    MasterKey,
    RestApiKey, // Or JavaScriptKey if REST API Key is not set
    NoAuth,     // For public readable data or specific endpoints like login/signup
}

#[derive(Debug, Clone)]
pub struct ParseClient {
    pub server_url: String, // Changed from Url to String
    pub(crate) app_id: String,
    #[allow(dead_code)] // Not used by current auth features
    pub(crate) javascript_key: Option<String>,
    pub(crate) rest_api_key: Option<String>,
    pub(crate) master_key: Option<String>,
    pub(crate) http_client: Client, // Updated to use alias
    pub(crate) session_token: Option<String>,
}

impl ParseClient {
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
            "ParseClient initialized with base server_url: {}",
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

    fn _set_session_token(&mut self, token: Option<String>) {
        self.session_token = token;
    }

    /// Returns the current session token, if any.
    pub fn session_token(&self) -> Option<&str> {
        self.session_token.as_deref()
    }

    /// Checks if the client is currently authenticated (has a session token).
    pub fn is_authenticated(&self) -> bool {
        self.session_token.is_some()
    }

    /// Uploads a file to the Parse Server.
    ///
    /// # Arguments
    /// * `file_name`: The desired name for the file on the server.
    /// * `data`: The raw byte data of the file.
    /// * `mime_type`: The MIME type of the file (e.g., "image/jpeg").
    ///
    /// # Returns
    /// A `Result` containing a `FileField` struct with the name and URL of the uploaded file,
    /// or a `ParseError` if the upload fails.
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
        log::debug!("--- ParseClient: Uploading File ---");
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

    /// Updates the schema for a given class, typically to set Class-Level Permissions.
    /// Requires the Master Key.
    ///
    /// # Arguments
    /// * `class_name`: The name of the class whose schema is to be updated.
    /// * `schema_payload`: A `serde_json::Value` representing the schema update payload (e.g., CLPs).
    ///
    /// # Returns
    /// A `Result` containing the server's response (typically the updated schema definition as `serde_json::Value`) or a `ParseError`.
    pub async fn update_class_schema(
        &self,
        class_name: &str,
        schema_payload: &Value,
    ) -> Result<Value, ParseError> {
        if self.master_key.is_none() {
            return Err(ParseError::MasterKeyRequired(
                "Master key is required to update class schema.".to_string(),
            ));
        }
        let endpoint = format!("schemas/{}", class_name);
        self._request(Method::PUT, &endpoint, Some(schema_payload), true, None) // Pass relative endpoint
            .await
    }

    // Methods to get handles for specific Parse features
    pub fn user(&mut self) -> ParseUserHandle<'_> {
        ParseUserHandle::new(self)
    }

    pub fn session(&self) -> crate::session::ParseSessionHandle<'_> {
        crate::session::ParseSessionHandle::new(self)
    }

    pub fn cloud(&self) -> ParseCloud<'_> {
        ParseCloud::new(self)
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
impl ParseClient {
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
            log::debug!("--- ParseClient GET Request --- ");
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
            log::debug!("--- ParseClient Response ---");
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
