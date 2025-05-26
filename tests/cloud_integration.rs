// tests/cloud_integration.rs

use serde_json::{json, Value};

mod query_test_utils;

use query_test_utils::shared::setup_client_with_master_key;

// Placeholder for a test that calls a simple cloud function (e.g., "hello")
// This assumes a cloud function named "hello" is defined on the test Parse Server
// that returns { "result": "Hello from Cloud Code!" }
#[tokio::test]
async fn test_call_simple_cloud_function() {
    let client = setup_client_with_master_key();
    // Define expected result type if known, or use serde_json::Value
    let result: Result<String, parse_rs::ParseError> = client
        .cloud()
        .run("hello", &json!({})) // Empty params
        .await;

    match result {
        Ok(message) => {
            assert_eq!(message, "Hello from Cloud Code!"); // Adjust expected message as per your cloud function
        }
        Err(e) => {
            panic!("Cloud function call failed: {:?}", e);
        }
    }
}

// Placeholder for a test that calls a cloud function with parameters
// This assumes a cloud function named "echo" is defined on the test Parse Server
// that accepts a parameter (e.g., "message") and returns it:
// { "result": { "echoedMessage": params.message } }
#[tokio::test]
async fn test_call_cloud_function_with_params() {
    let client = setup_client_with_master_key();
    let params = json!({ "message": "Test message" });

    #[derive(serde::Deserialize, Debug, PartialEq)]
    struct EchoResponse {
        #[serde(rename = "echoedMessage")] // Assuming the cloud function returns this field name
        echoed_message: String,
    }

    let result: Result<EchoResponse, parse_rs::ParseError> =
        client.cloud().run("echo", &params).await;

    match result {
        Ok(response) => {
            assert_eq!(response.echoed_message, "Test message");
        }
        Err(e) => {
            panic!("Cloud function call with params failed: {:?}", e);
        }
    }
}

// Placeholder for a test that calls a non-existent cloud function
#[tokio::test]
async fn test_call_non_existent_cloud_function() {
    let client = setup_client_with_master_key();
    let params = json!({});

    let result: Result<Value, parse_rs::ParseError> =
        client.cloud().run("nonExistentFunction", &params).await;

    match result {
        Ok(_) => panic!("Calling a non-existent function should fail"),
        Err(parse_rs::ParseError::OtherParseError { code, message }) => {
            assert_eq!(
                code, 141,
                "Expected error code 141 for non-existent function, got {}",
                code
            );
            assert!(
                message.contains("Invalid function: \"nonExistentFunction\""),
                "Error message should indicate invalid function. Got: {}",
                message
            );
        }
        Err(e) => {
            panic!(
                "Unexpected error type when calling non-existent function: {:?}",
                e
            );
        }
    }
}
