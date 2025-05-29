// src/cloud.rs

use crate::{client::Parse, error::ParseError};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// Internal helper struct to deserialize the `{"result": ...}` wrapper from Parse Cloud Function responses.
///
/// Parse Server wraps the actual return value of a cloud function within a JSON object under the key `"result"`.
/// This struct facilitates deserializing that wrapper to extract the actual expected type `T`.
#[derive(Deserialize, Debug)]
struct CloudFunctionResponse<T> {
    result: T,
}

/// Provides methods for interacting with Parse Cloud Code functions.
///
/// An instance of `ParseCloud` is obtained by calling the [`cloud()`](crate::Parse::cloud)
/// method on a `Parse` instance. It allows for executing server-side Cloud Code functions,
/// passing parameters, and receiving their results.
///
/// Cloud Code functions are custom JavaScript (or other supported language) functions deployed to your
/// Parse Server, enabling server-side logic, data validation, triggers, and more, without exposing
/// sensitive operations or master key usage directly to the client.
///
/// This handle operates in the context of the `Parse` it was created from, using its configuration
/// (server URL, app ID, keys, and current session token if any) for API requests to the `/functions` endpoint.
#[derive(Debug)]
pub struct ParseCloud<'a> {
    client: &'a Parse,
}

impl<'a> ParseCloud<'a> {
    /// Creates a new `ParseCloud` handler.
    pub(crate) fn new(client: &'a Parse) -> Self {
        ParseCloud { client }
    }

    /// Runs a Parse Cloud Function and returns its result.
    ///
    /// This method sends a POST request to the `/functions/:functionName` endpoint, where
    /// `:functionName` is the name of the Cloud Code function to execute. The `params` argument
    /// is serialized to JSON and sent as the request body.
    ///
    /// The Parse Server executes the specified function and is expected to return a JSON object
    /// of the form `{"result": ...}`, where `...` is the actual value returned by the function.
    /// This method automatically unwraps the `result` field and deserializes its content into
    /// the type `R` specified by the caller.
    ///
    /// # Type Parameters
    ///
    /// * `P`: The type of the `params` argument. This type must implement `Serialize`, `Send`, and `Sync`.
    ///   It can be any serializable type, such as a custom struct, `serde_json::Value`, or a `HashMap`.
    /// * `R`: The expected type of the `result` field from the cloud function's response. This type
    ///   must implement `DeserializeOwned`, `Send`, `Sync`, and be `'static`.
    ///
    /// # Arguments
    ///
    /// * `function_name`: A string slice representing the name of the cloud function to execute.
    /// * `params`: A reference to the parameters to pass to the cloud function.
    ///
    /// # Returns
    ///
    /// A `Result` containing the deserialized value from the `result` field of the cloud function's
    /// response if successful. Returns a `ParseError` if the function name is invalid, parameters
    /// cannot be serialized, the server returns an error (e.g., function not found, internal error
    /// in cloud code), the response cannot be deserialized into `R`, or any other network/request error occurs.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use parse_rs::{Parse, ParseError};
    /// use serde::{Serialize, Deserialize};
    /// use serde_json::json; // For ad-hoc JSON parameters
    ///
    /// // Define a struct for expected parameters if your function takes structured input
    /// #[derive(Serialize)]
    /// struct HelloParams<'a> {
    ///     name: &'a str,
    /// }
    ///
    /// // Define a struct for the expected result if your function returns structured data
    /// #[derive(Deserialize, Debug)]
    /// struct HelloResponse {
    ///     message: String,
    ///     timestamp: u64,
    /// }
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), ParseError> {
    /// # let server_url = std::env::var("PARSE_SERVER_URL").unwrap_or_else(|_| "http://localhost:1338/parse".to_string());
    /// # let app_id = std::env::var("PARSE_APP_ID").unwrap_or_else(|_| "myAppId".to_string());
    /// # let client = Parse::new(&server_url, &app_id, None, None, None)?;
    ///
    /// // Example 1: Calling a cloud function named "hello" with simple JSON parameters
    /// // and expecting a simple string response.
    /// // Assume cloud function "hello" is defined as: Parse.Cloud.define("hello", req => `Hello, ${req.params.name}!`);
    /// let params1 = json!({ "name": "World" });
    /// match client.cloud().run::<_, String>("hello", &params1).await {
    ///     Ok(response_message) => {
    ///         println!("Cloud function 'hello' responded: {}", response_message);
    ///         assert_eq!(response_message, "Hello, World!");
    ///     }
    ///     Err(e) => eprintln!("Failed to run cloud function 'hello': {}", e),
    /// }
    ///
    /// // Example 2: Calling a cloud function "complexHello" with structured parameters
    /// // and expecting a structured response.
    /// // Assume cloud function "complexHello" is defined as:
    /// // Parse.Cloud.define("complexHello", req => {
    /// //   return { message: `Complex hello to ${req.params.name}!`, timestamp: Date.now() };
    /// // });
    /// let params2 = HelloParams { name: "SDK User" };
    /// match client.cloud().run::<HelloParams, HelloResponse>("complexHello", &params2).await {
    ///     Ok(response_data) => {
    ///         println!(
    ///             "Cloud function 'complexHello' responded with message: '{}' at {}",
    ///             response_data.message,
    ///             response_data.timestamp
    ///         );
    ///         assert!(response_data.message.contains("SDK User"));
    ///     }
    ///     Err(e) => eprintln!("Failed to run cloud function 'complexHello': {}", e),
    /// }
    ///
    /// // Example 3: Calling a cloud function that takes no parameters and returns a number
    /// // Assume cloud function "randomNumber" is defined as: Parse.Cloud.define("randomNumber", () => Math.random() * 100);
    /// let empty_params = json!({}); // Or use a unit type `()` if your function truly takes no params and server handles it.
    /// match client.cloud().run::<_, f64>("randomNumber", &empty_params).await {
    ///     Ok(number) => {
    ///         println!("Cloud function 'randomNumber' responded: {}", number);
    ///         assert!(number >= 0.0 && number <= 100.0);
    ///     }
    ///     Err(e) => eprintln!("Failed to run cloud function 'randomNumber': {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run<P, R>(&self, function_name: &str, params: &P) -> Result<R, ParseError>
    where
        P: Serialize + Send + Sync,
        R: DeserializeOwned + Send + Sync + 'static,
    {
        let endpoint = format!("functions/{}", function_name);
        let response_wrapper: CloudFunctionResponse<R> =
            self.client.post(&endpoint, params).await?;
        Ok(response_wrapper.result)
    }

    // Note: Background jobs are triggered via /parse/jobs endpoint and typically require MasterKey.
    // This could be a separate method `trigger_job` if needed in the future.
}
