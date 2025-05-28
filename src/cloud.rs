// src/cloud.rs

use crate::{client::ParseClient, error::ParseError};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// Represents the response from a Parse Cloud Function call.
#[derive(Deserialize, Debug)]
struct CloudFunctionResponse<T> {
    result: T,
}

/// Provides methods for interacting with Parse Cloud Code functions.
///
/// An instance of `ParseCloud` is obtained by calling the [`cloud()`](crate::ParseClient::cloud)
/// method on a `ParseClient` instance.
#[derive(Debug)]
pub struct ParseCloud<'a> {
    client: &'a ParseClient,
}

impl<'a> ParseCloud<'a> {
    /// Creates a new `ParseCloud` handler.
    pub(crate) fn new(client: &'a ParseClient) -> Self {
        ParseCloud { client }
    }

    /// Runs a Parse Cloud Function.
    ///
    /// # Arguments
    ///
    /// * `function_name`: The name of the cloud function to execute.
    /// * `params`: The parameters to pass to the cloud function. This must be a type
    ///   that can be serialized into JSON (e.g., `serde_json::Value`, a struct, a map).
    ///
    /// # Returns
    ///
    /// A `Result` containing the deserialized `result` field from the cloud function's
    /// response, or a `ParseError` if the operation fails.
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
