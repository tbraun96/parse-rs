// tests/installation_query_integration.rs

use parse_rs::installation::{DeviceType, NewParseInstallation, RetrievedParseInstallation};
use parse_rs::Parse;
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

// Assuming query_test_utils provides a setup_client_with_master_key function
mod query_test_utils;
use query_test_utils::shared::setup_client_with_master_key;

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a unique installation for testing
    async fn create_test_installation(
        client: &Parse,
        device_type: DeviceType,
        installation_id_suffix: &str,
        channels: Option<Vec<String>>,
        custom_field_value: Option<&str>,
        test_run_id: Option<&str>,
    ) -> RetrievedParseInstallation {
        let unique_installation_id = format!(
            "test_install_id_{}_{}",
            installation_id_suffix,
            Uuid::new_v4().simple()
        );
        let mut new_installation = NewParseInstallation::new(device_type.clone());
        new_installation.installation_id = Some(unique_installation_id.clone());
        new_installation.app_name = Some("Parse Rust SDK Query Test".to_string());
        new_installation.device_token =
            Some(format!("query_test_token_{}", Uuid::new_v4().simple()));
        new_installation.channels =
            channels.or_else(|| Some(vec![format!("query_channel_{}", Uuid::new_v4().simple())]));

        let mut custom_fields = HashMap::new();
        if let Some(value) = custom_field_value {
            custom_fields.insert("customQueryField".to_string(), json!(value));
        }
        if let Some(id) = test_run_id {
            custom_fields.insert("testRunId".to_string(), json!(id));
        }
        if !custom_fields.is_empty() {
            new_installation.custom_fields = Some(custom_fields);
        }

        let created = client
            .create_installation(&new_installation)
            .await
            .expect("Failed to create test installation");

        // Retrieve the full installation object to ensure all fields are present for assertions later
        client
            .get_installation(&created.object_id)
            .await
            .expect("Failed to retrieve created test installation")
    }

    // Tests will be added here in the next step

    #[tokio::test]
    async fn test_query_installation_by_device_type() {
        let client = setup_client_with_master_key();
        let unique_test_run_id = format!("run_{}", Uuid::new_v4().simple());

        // Create an iOS installation with the unique test run ID
        let ios_installation = create_test_installation(
            &client,
            DeviceType::Ios,
            "ios_device",
            None,
            None,
            Some(&unique_test_run_id),
        )
        .await;
        // Create an Android installation (without the specific test run ID, or with a different one if needed for other tests)
        let _android_installation = create_test_installation(
            &client,
            DeviceType::Android,
            "android_device",
            None,
            None,
            None,
        )
        .await;

        let mut query = client.query_installations();
        query.equal_to("deviceType", "ios");
        query.equal_to("testRunId", &unique_test_run_id);

        let results: Result<Vec<RetrievedParseInstallation>, _> = query.find(&client).await;

        assert!(results.is_ok(), "Query failed: {:?}", results.err());
        let installations = results.unwrap();

        assert_eq!(
            installations.len(),
            1,
            "Expected to find 1 iOS installation, found {}",
            installations.len()
        );
        assert_eq!(
            installations[0].object_id, ios_installation.object_id,
            "Found installation does not match the created iOS installation"
        );
        assert_eq!(
            installations[0].device_type,
            DeviceType::Ios,
            "Found installation device_type is not Ios"
        );

        // Cleanup: Delete created installations
        client
            .delete_installation(&ios_installation.object_id)
            .await
            .expect("Failed to delete iOS installation");
        client
            .delete_installation(&_android_installation.object_id)
            .await
            .expect("Failed to delete android installation");
    }

    #[tokio::test]
    async fn test_query_installation_by_custom_field() {
        let client = setup_client_with_master_key();
        let unique_test_run_id_a = format!("run_A_{}", Uuid::new_v4().simple());
        let unique_test_run_id_b = format!("run_B_{}", Uuid::new_v4().simple());

        // Create installation A with customQueryField: "valueA"
        let installation_a = create_test_installation(
            &client,
            DeviceType::Js,
            "custom_A",
            None,
            Some("valueA"),
            Some(&unique_test_run_id_a),
        )
        .await;
        // Create installation B with customQueryField: "valueB"
        let installation_b = create_test_installation(
            &client,
            DeviceType::Js,
            "custom_B",
            None,
            Some("valueB"),
            Some(&unique_test_run_id_b),
        )
        .await;

        let mut query = client.query_installations();
        query.equal_to("customQueryField", "valueA");
        query.equal_to("testRunId", &unique_test_run_id_a);

        let results: Result<Vec<RetrievedParseInstallation>, _> = query.find(&client).await;

        assert!(results.is_ok(), "Query failed: {:?}", results.err());
        let installations = results.unwrap();

        assert_eq!(
            installations.len(),
            1,
            "Expected to find 1 installation with customQueryField 'valueA', found {}",
            installations.len()
        );
        assert_eq!(
            installations[0].object_id, installation_a.object_id,
            "Found installation does not match installation_a"
        );
        assert_eq!(
            installations[0]
                .custom_fields
                .get("customQueryField")
                .and_then(|v| v.as_str()),
            Some("valueA")
        );
        assert_eq!(
            installations[0]
                .custom_fields
                .get("testRunId")
                .and_then(|v| v.as_str()),
            Some(unique_test_run_id_a.as_str())
        );

        // Cleanup
        client
            .delete_installation(&installation_a.object_id)
            .await
            .expect("Failed to delete installation_a");
        client
            .delete_installation(&installation_b.object_id)
            .await
            .expect("Failed to delete installation_b");
    }

    #[tokio::test]
    async fn test_query_installation_by_installation_id() {
        let client = setup_client_with_master_key();
        let target_installation_id = format!("target_install_id_{}", Uuid::new_v4().simple());
        let other_installation_id = format!("other_install_id_{}", Uuid::new_v4().simple());
        let unique_test_run_id = format!("run_install_id_query_{}", Uuid::new_v4().simple());

        // Create the target installation
        let mut target_install_data = NewParseInstallation::new(DeviceType::Android);
        target_install_data.installation_id = Some(target_installation_id.clone());
        target_install_data.app_name = Some("Query Test App".to_string());
        let mut custom_fields = HashMap::new();
        custom_fields.insert("testRunId".to_string(), json!(&unique_test_run_id));
        target_install_data.custom_fields = Some(custom_fields);
        let target_created = client
            .create_installation(&target_install_data)
            .await
            .expect("Failed to create target installation");

        // Create another installation to ensure query is specific
        let mut other_install_data = NewParseInstallation::new(DeviceType::Android);
        other_install_data.installation_id = Some(other_installation_id.clone());
        // No testRunId for this one, or a different one, to ensure it's not picked up by the main query
        let other_created = client
            .create_installation(&other_install_data)
            .await
            .expect("Failed to create other installation");

        let mut query = client.query_installations();
        query.equal_to("installationId", &target_installation_id);
        query.equal_to("testRunId", &unique_test_run_id); // Ensure we get THIS test's installation

        let results: Result<Vec<RetrievedParseInstallation>, _> = query.find(&client).await;

        assert!(results.is_ok(), "Query failed: {:?}", results.err());
        let installations = results.unwrap();

        assert_eq!(
            installations.len(),
            1,
            "Expected to find 1 installation with installationId '{}', found {}",
            target_installation_id,
            installations.len()
        );
        assert_eq!(
            installations[0].object_id, target_created.object_id,
            "Found installation does not match the target created installation"
        );
        assert_eq!(
            installations[0].installation_id.as_deref(),
            Some(target_installation_id.as_str())
        );
        assert_eq!(
            installations[0]
                .custom_fields
                .get("testRunId")
                .and_then(|v| v.as_str()),
            Some(unique_test_run_id.as_str())
        );

        // Cleanup
        client
            .delete_installation(&target_created.object_id)
            .await
            .expect("Failed to delete target installation");
        client
            .delete_installation(&other_created.object_id)
            .await
            .expect("Failed to delete other installation");
    }
}
