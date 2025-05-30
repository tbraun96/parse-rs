// tests/installation_integration.rs
use parse_rs::installation::{DeviceType, NewParseInstallation, UpdateParseInstallation};
use uuid::Uuid;

mod query_test_utils; // Assuming this contains setup_client_with_master_key
use query_test_utils::shared::setup_client_with_master_key;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_installation() {
        let client = setup_client_with_master_key();
        let unique_installation_id = Uuid::new_v4().to_string();

        let mut new_installation = NewParseInstallation::new(DeviceType::Js);
        new_installation.installation_id = Some(unique_installation_id.clone());
        new_installation.app_name = Some("Parse Rust SDK Test".to_string());
        new_installation.app_version = Some("0.1.0".to_string());
        new_installation.device_token = Some(format!("test_token_{}", Uuid::new_v4().simple()));
        new_installation.channels = Some(vec!["test_channel".to_string()]);

        let result = client.create_installation(&new_installation).await;

        assert!(
            result.is_ok(),
            "Failed to create installation: {:?}",
            result.err()
        );
        let response = result.unwrap();
        assert!(
            !response.object_id.is_empty(),
            "ObjectId should not be empty"
        );
        assert!(
            !response.created_at.to_string().is_empty(),
            "CreatedAt should not be empty"
        );

        // TODO: Add cleanup logic if possible (delete the created installation)
        // For now, we'll rely on manual cleanup or a Parse Server that's reset for tests.
    }

    #[tokio::test]
    async fn test_create_installation_android_with_token() {
        let client = setup_client_with_master_key();
        let device_token = format!("android_test_token_{}", Uuid::new_v4().simple());

        let mut new_installation = NewParseInstallation::new(DeviceType::Android);
        new_installation.device_token = Some(device_token.clone());
        new_installation.app_identifier = Some("com.example.testapp".to_string());

        let result = client.create_installation(&new_installation).await;
        assert!(
            result.is_ok(),
            "Failed to create Android installation: {:?}",
            result.err()
        );
        let response = result.unwrap();
        assert!(!response.object_id.is_empty());

        // TODO: Optionally retrieve and verify fields if a get_installation method exists
    }

    #[tokio::test]
    async fn test_get_installation() {
        let client = setup_client_with_master_key();
        let unique_installation_id = Uuid::new_v4().to_string();
        let device_token_val = format!("get_test_token_{}", Uuid::new_v4().simple());

        let mut new_installation = NewParseInstallation::new(DeviceType::Ios);
        new_installation.installation_id = Some(unique_installation_id.clone());
        new_installation.device_token = Some(device_token_val.clone());
        new_installation.app_name = Some("Get Test App".to_string());
        new_installation.channels =
            Some(vec!["general_get".to_string(), "sdk_test_get".to_string()]);

        // Create the installation first
        let create_result = client.create_installation(&new_installation).await;
        assert!(
            create_result.is_ok(),
            "Failed to create installation for get test: {:?}",
            create_result.err()
        );
        let created_installation = create_result.unwrap();
        let object_id = created_installation.object_id;

        // Now try to get it
        let get_result = client.get_installation(&object_id).await;
        assert!(
            get_result.is_ok(),
            "Failed to get installation: {:?}",
            get_result.err()
        );
        let retrieved_installation = get_result.unwrap();

        assert_eq!(retrieved_installation.object_id, object_id);
        assert_eq!(
            retrieved_installation.installation_id.as_ref(),
            Some(&unique_installation_id)
        );
        assert_eq!(retrieved_installation.device_type, DeviceType::Ios);
        assert_eq!(
            retrieved_installation.device_token.as_ref(),
            Some(&device_token_val)
        );
        assert_eq!(
            retrieved_installation.app_name.as_ref(),
            Some(&"Get Test App".to_string())
        );

        let mut expected_channels = vec!["general_get".to_string(), "sdk_test_get".to_string()];
        expected_channels.sort(); // Sort for consistent comparison
        let mut actual_channels = retrieved_installation.channels.unwrap_or_default();
        actual_channels.sort();
        assert_eq!(actual_channels, expected_channels);

        // Check for a custom field if we had added one (example)
        // assert_eq!(retrieved_installation.custom_fields.get("custom_key").unwrap().as_str().unwrap(), "custom_value");
    }

    #[tokio::test]
    async fn test_update_installation() {
        let client = setup_client_with_master_key();
        let initial_installation_id = Uuid::new_v4().to_string();

        // 1. Create an installation
        let mut new_installation = NewParseInstallation::new(DeviceType::Android);
        new_installation.installation_id = Some(initial_installation_id.clone());
        new_installation.app_name = Some("Update Test App".to_string());
        new_installation.channels = Some(vec!["initial_channel".to_string()]);
        new_installation
            .custom_fields
            .get_or_insert_with(std::collections::HashMap::new)
            .insert("custom_status".to_string(), serde_json::json!("active"));

        let create_result = client.create_installation(&new_installation).await;
        assert!(
            create_result.is_ok(),
            "Failed to create installation for update test: {:?}",
            create_result.err()
        );
        let created_object_id = create_result.unwrap().object_id;

        // 2. Update the installation
        let mut updates = UpdateParseInstallation::new();
        updates.badge = Some(5);
        updates.channels = Some(vec![
            "updated_channel_1".to_string(),
            "updated_channel_2".to_string(),
        ]);
        updates.app_version = Some("1.1.0".to_string());
        updates
            .custom_fields
            .insert("custom_status".to_string(), serde_json::json!("inactive"));
        updates
            .custom_fields
            .insert("new_custom_field".to_string(), serde_json::json!(true));

        let update_result = client
            .update_installation(&created_object_id, &updates)
            .await;
        assert!(
            update_result.is_ok(),
            "Failed to update installation: {:?}",
            update_result.err()
        );
        let update_response = update_result.unwrap();
        assert!(
            !update_response.updated_at.iso.is_empty(),
            "UpdatedAt string in response should not be empty"
        );

        // 3. Retrieve the installation to verify updates
        let get_result = client.get_installation(&created_object_id).await;
        assert!(
            get_result.is_ok(),
            "Failed to get updated installation: {:?}",
            get_result.err()
        );
        let retrieved_installation = get_result.unwrap();

        // 4. Verify fields
        assert_eq!(retrieved_installation.badge, Some(5));
        assert_eq!(
            retrieved_installation.app_version.as_ref(),
            Some(&"1.1.0".to_string())
        );

        let mut expected_channels = vec![
            "updated_channel_1".to_string(),
            "updated_channel_2".to_string(),
        ];
        expected_channels.sort();
        let mut actual_channels = retrieved_installation.channels.unwrap_or_default();
        actual_channels.sort();
        assert_eq!(actual_channels, expected_channels);

        assert_eq!(
            retrieved_installation
                .custom_fields
                .get("custom_status")
                .unwrap(),
            &serde_json::json!("inactive")
        );
        assert_eq!(
            retrieved_installation
                .custom_fields
                .get("new_custom_field")
                .unwrap(),
            &serde_json::json!(true)
        );

        // Ensure original fields not part of the update remain (e.g., appName)
        assert_eq!(
            retrieved_installation.app_name.as_ref(),
            Some(&"Update Test App".to_string())
        );
        // Ensure installationId (which is usually immutable or set at creation) is still the same
        assert_eq!(
            retrieved_installation.installation_id.as_ref(),
            Some(&initial_installation_id)
        );
    }

    #[tokio::test]
    async fn test_delete_installation() {
        let client = setup_client_with_master_key();
        let installation_id_val = Uuid::new_v4().to_string();

        // 1. Create an installation
        let mut new_installation = NewParseInstallation::new(DeviceType::Js);
        new_installation.installation_id = Some(installation_id_val.clone());
        new_installation.app_name = Some("Delete Test App".to_string());

        let create_result = client.create_installation(&new_installation).await;
        assert!(
            create_result.is_ok(),
            "Failed to create installation for delete test: {:?}",
            create_result.err()
        );
        let created_object_id = create_result.unwrap().object_id;

        // 2. Delete the installation
        let delete_result = client.delete_installation(&created_object_id).await;
        assert!(
            delete_result.is_ok(),
            "Failed to delete installation: {:?}",
            delete_result.err()
        );

        // 3. Attempt to retrieve the deleted installation
        let get_result = client.get_installation(&created_object_id).await;
        assert!(
            get_result.is_err(),
            "Should not be able to retrieve a deleted installation."
        );

        if let Err(parse_rs::ParseError::ObjectNotFound(error_message)) = get_result {
            // Check if the error message contains the expected text
            assert!(
                error_message.contains("(101) Object not found")
                    || error_message.contains("(101) object not found"),
                "Error message mismatch: {}",
                error_message
            );
        } else {
            panic!(
                "Expected ObjectNotFound error, but got: {:?}",
                get_result.err()
            );
        }
    }
}
