use parse_rs::ParseConfig; // Keep Parse for query_test_utils, ParseError for expect/unwrap
use serde_json::{json, Value};
use std::collections::HashMap;
use uuid::Uuid;

mod query_test_utils;
use query_test_utils::shared::setup_client_with_master_key;

// Helper to safely get a string value from config or default
fn get_param_value_or_null(config: &ParseConfig, param_name: &str) -> Value {
    config.get::<Value>(param_name).unwrap_or(Value::Null)
}

#[tokio::test]
async fn test_get_and_update_config() {
    let client = setup_client_with_master_key();

    // 1. Get initial configuration (we don't care if it's empty or not initially)
    let initial_config_response = client.get_config().await;
    assert!(
        initial_config_response.is_ok(),
        "Failed to get initial config: {:?}",
        initial_config_response.err()
    );
    let initial_config = initial_config_response.unwrap();
    // No assertion on initial_config.params.is_empty() anymore

    // Define a test parameter that is unlikely to exist
    let test_param_name = format!("sdkTestParam_{}", Uuid::new_v4().simple());
    let original_value_before_test = get_param_value_or_null(&initial_config, &test_param_name);
    let new_value = json!("sdk_test_value_789");

    // Ensure our test parameter is different from what might be there
    assert_ne!(
        original_value_before_test, new_value,
        "Test value is somehow already set to new_value"
    );

    // 2. Update the configuration with our test parameter
    let mut params_to_update = HashMap::new();
    params_to_update.insert(test_param_name.clone(), new_value.clone());

    let update_response = client.update_config(&params_to_update).await;
    assert!(
        update_response.is_ok(),
        "Failed to update config with test param: {:?}",
        update_response.err()
    );
    assert!(
        update_response.unwrap().result,
        "Update config with test param result should be true"
    );

    // 3. Get configuration again to verify the update
    let updated_config_response = client.get_config().await;
    assert!(
        updated_config_response.is_ok(),
        "Failed to get updated config: {:?}",
        updated_config_response.err()
    );
    let updated_config = updated_config_response.unwrap();

    let current_test_param_value = updated_config
        .get::<Value>(&test_param_name)
        .expect("Test parameter should exist after update");
    assert_eq!(
        current_test_param_value, new_value,
        "Test parameter was not updated correctly"
    );

    // 4. Revert the parameter to its original state (which might be removing it if it was Value::Null)
    let mut params_to_revert = HashMap::new();
    params_to_revert.insert(test_param_name.clone(), original_value_before_test.clone());

    let revert_response = client.update_config(&params_to_revert).await;
    assert!(
        revert_response.is_ok(),
        "Failed to revert config: {:?}",
        revert_response.err()
    );
    assert!(
        revert_response.unwrap().result,
        "Revert config result should be true"
    );

    // 5. Final check to ensure reversion
    let final_config_response = client.get_config().await;
    assert!(
        final_config_response.is_ok(),
        "Failed to get final config: {:?}",
        final_config_response.err()
    );
    let final_config = final_config_response.unwrap();
    let final_test_param_value = get_param_value_or_null(&final_config, &test_param_name);
    assert_eq!(
        final_test_param_value, original_value_before_test,
        "Test parameter was not reverted correctly"
    );

    println!("ParseConfig integration test completed successfully.");
}
