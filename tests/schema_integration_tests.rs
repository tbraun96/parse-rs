use parse_rs::schema::{ClassLevelPermissionsSchema, FieldSchema, FieldType, IndexFieldType};
use parse_rs::ParseError;
use serde_json::json;
use std::collections::HashMap;

mod query_test_utils;
use query_test_utils::shared::setup_client_with_master_key;

// Helper function to generate a unique class name for testing to avoid collisions
fn unique_class_name(base: &str) -> String {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    let suffix: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();
    format!("{}_{}", base, suffix)
}

#[tokio::test]
async fn test_create_get_and_delete_class_schema() {
    let client = setup_client_with_master_key();
    let class_name = unique_class_name("TestSchemaLifecycle");

    // 1. Create a new class schema
    let mut fields_to_create = HashMap::new();
    fields_to_create.insert(
        "textField".to_string(),
        FieldSchema {
            field_type: FieldType::String,
            target_class: None,
            required: Some(false),
            default_value: Some(json!("default text")),
        },
    );
    fields_to_create.insert(
        "numberField".to_string(),
        FieldSchema {
            field_type: FieldType::Number,
            target_class: None,
            required: Some(true),
            default_value: None,
        },
    );

    let clp = ClassLevelPermissionsSchema {
        find: Some([("*".to_string(), true)].iter().cloned().collect()),
        get: Some([("*".to_string(), true)].iter().cloned().collect()),
        create: Some([("*".to_string(), true)].iter().cloned().collect()),
        update: Some([("*".to_string(), true)].iter().cloned().collect()),
        delete: Some([("*".to_string(), true)].iter().cloned().collect()),
        add_field: Some([("*".to_string(), true)].iter().cloned().collect()),
        ..Default::default()
    };

    let schema_to_create_payload = json!({
        "className": class_name,
        "fields": fields_to_create,
        "classLevelPermissions": clp
    });

    let created_schema = client
        .create_class_schema(&class_name, &schema_to_create_payload)
        .await
        .expect("Failed to create class schema");

    assert_eq!(created_schema.class_name, class_name);
    assert_eq!(created_schema.fields.len(), fields_to_create.len() + 4); // +3 for default ACL, objectId, createdAt, updatedAt (actually +4, server adds them)
    assert!(created_schema.fields.contains_key("textField"));
    assert!(created_schema.fields.contains_key("numberField"));
    assert!(created_schema.fields.contains_key("ACL"));
    assert!(created_schema.fields.contains_key("objectId"));
    assert!(created_schema.fields.contains_key("createdAt"));
    assert!(created_schema.fields.contains_key("updatedAt"));

    if let Some(retrieved_clp) = &created_schema.class_level_permissions {
        assert_eq!(retrieved_clp.find, clp.find);
    } else {
        panic!("CLP not found in created schema response");
    }

    // 2. Verify by fetching the specific schema
    let fetched_schema = client
        .get_class_schema(&class_name)
        .await
        .expect("Failed to fetch created class schema");

    assert_eq!(fetched_schema.class_name, class_name);
    assert_eq!(fetched_schema.fields.len(), created_schema.fields.len());
    assert_eq!(
        fetched_schema.class_level_permissions,
        created_schema.class_level_permissions
    );

    // 3. Verify it appears in all schemas list
    let all_schemas_response = client
        .get_all_schemas()
        .await
        .expect("Failed to get all schemas");

    // Debug print for indexes
    for schema in &all_schemas_response.results {
        if let Some(indexes) = &schema.indexes {
            for (index_name, index_fields) in indexes {
                for (field_name, value) in index_fields {
                    match value {
                        IndexFieldType::Text(text_val) => {
                            println!(
                                "DEBUG: Schema '{}', Index '{}', Field '{}': Text index value: {:?}",
                                schema.class_name, index_name, field_name, text_val
                            );
                        }
                        IndexFieldType::Other(other_val) => {
                            println!(
                                "DEBUG: Schema '{}', Index '{}', Field '{}': Other index value: {:?}",
                                schema.class_name, index_name, field_name, other_val
                            );
                        }
                        IndexFieldType::SortOrder(_) => {
                            // This is expected, do nothing.
                        }
                    }
                }
            }
        }
    }

    assert!(
        all_schemas_response
            .results
            .iter()
            .any(|s| s.class_name == class_name),
        "Created schema not found in all schemas list"
    );

    // 4. Delete the class schema
    client
        .delete_class_schema(&class_name, true)
        .await
        .expect("Failed to delete class schema");

    // 5. Verify deletion by trying to fetch it again (should fail)
    match client.get_class_schema(&class_name).await {
        Err(ParseError::OtherParseError { code, message }) => {
            assert_eq!(code, 103);
            assert!(
                message.to_lowercase().contains("does not exist")
                    || message.to_lowercase().contains("class not found")
            );
        }
        Ok(_) => panic!("Fetching schema after deletion should have failed but succeeded."),
        Err(e) => panic!(
            "Fetching schema after deletion failed with an unexpected error: {}",
            e
        ),
    }

    // Also check it's not in all_schemas list anymore
    let all_schemas_after_delete = client
        .get_all_schemas()
        .await
        .expect("Failed to get all schemas after delete");
    assert!(
        !all_schemas_after_delete
            .results
            .iter()
            .any(|s| s.class_name == class_name),
        "Deleted schema still found in all schemas list"
    );
}

#[tokio::test]
async fn test_update_class_schema() {
    let client = setup_client_with_master_key();
    let class_name = unique_class_name("TestSchemaUpdate");

    // 1. Create an initial class schema
    let mut initial_fields = HashMap::new();
    initial_fields.insert(
        "fieldToDelete".to_string(),
        FieldSchema {
            field_type: FieldType::String,
            target_class: None,
            required: Some(false),
            default_value: None,
        },
    );
    initial_fields.insert(
        "fieldToKeep".to_string(),
        FieldSchema {
            field_type: FieldType::Number,
            target_class: None,
            required: Some(false),
            default_value: Some(json!(100)),
        },
    );

    let initial_clp = ClassLevelPermissionsSchema {
        find: Some([("*".to_string(), true)].iter().cloned().collect()),
        get: Some([("*".to_string(), true)].iter().cloned().collect()),
        create: Some([("*".to_string(), true)].iter().cloned().collect()),
        update: Some([("*".to_string(), true)].iter().cloned().collect()),
        delete: Some([("*".to_string(), true)].iter().cloned().collect()),
        add_field: Some([("*".to_string(), true)].iter().cloned().collect()),
        ..Default::default()
    };

    let initial_schema_payload = json!({
        "className": class_name,
        "fields": initial_fields,
        "classLevelPermissions": initial_clp
    });

    let created_schema = client
        .create_class_schema(&class_name, &initial_schema_payload)
        .await
        .expect("Failed to create initial schema for update test");

    // Server adds 4 default fields: objectId, createdAt, updatedAt, ACL
    assert_eq!(
        created_schema.fields.len(),
        initial_fields.len() + 4,
        "Initial field count mismatch"
    );

    // 2. Update the schema
    let mut fields_update = HashMap::new();
    // Add a new field
    fields_update.insert(
        "addedField".to_string(),
        json!({
            "type": "Boolean",
            "required": true,
            "defaultValue": false
        }),
    );
    // Delete an existing field
    fields_update.insert("fieldToDelete".to_string(), json!({ "__op": "Delete" }));

    let updated_clp_map: HashMap<String, bool> =
        [("role:Admin".to_string(), true)].iter().cloned().collect();
    let updated_clp = ClassLevelPermissionsSchema {
        find: Some(updated_clp_map.clone()),
        get: Some(updated_clp_map.clone()),
        create: Some(updated_clp_map.clone()),
        update: Some(updated_clp_map.clone()),
        delete: Some(updated_clp_map.clone()),
        add_field: Some(updated_clp_map.clone()),
        ..Default::default()
    };

    let mut indexes_to_add = HashMap::new();
    let mut index_fields = HashMap::new();
    index_fields.insert("fieldToKeep".to_string(), 1); // 1 for ascending
    indexes_to_add.insert("idx_fieldToKeep".to_string(), index_fields);

    let schema_update_payload = json!({
        "className": class_name, // Must match class_name in path
        "fields": fields_update,
        "classLevelPermissions": updated_clp,
        "indexes": indexes_to_add
    });

    let updated_schema = client
        .update_class_schema(&class_name, &schema_update_payload)
        .await
        .expect("Failed to update class schema");

    // 3. Verify the updates
    assert_eq!(updated_schema.class_name, class_name);
    // Expected fields: fieldToKeep, addedField + 4 default fields (objectId, createdAt, updatedAt, ACL)
    // fieldToDelete was removed.
    assert_eq!(
        updated_schema.fields.len(),
        1 + 1 + 4,
        "Updated field count mismatch"
    );
    assert!(updated_schema.fields.contains_key("fieldToKeep"));
    assert!(updated_schema.fields.contains_key("addedField"));
    assert!(!updated_schema.fields.contains_key("fieldToDelete"));

    assert_eq!(
        updated_schema.fields.get("addedField").unwrap().field_type,
        FieldType::Boolean
    );
    assert_eq!(
        updated_schema.fields.get("addedField").unwrap().required,
        Some(true)
    );
    assert_eq!(
        updated_schema
            .fields
            .get("addedField")
            .unwrap()
            .default_value,
        Some(json!(false))
    );

    if let Some(clp_after_update) = &updated_schema.class_level_permissions {
        assert_eq!(
            clp_after_update.find, updated_clp.find,
            "CLP 'find' mismatch after update"
        );
        assert_eq!(
            clp_after_update.get, updated_clp.get,
            "CLP 'get' mismatch after update"
        );
    } else {
        panic!("CLP not found in updated schema response");
    }

    if let Some(indexes_after_update) = &updated_schema.indexes {
        assert!(
            indexes_after_update.contains_key("idx_fieldToKeep"),
            "Index not found after update"
        );
        assert_eq!(
            indexes_after_update
                .get("idx_fieldToKeep")
                .unwrap()
                .get("fieldToKeep"),
            Some(&IndexFieldType::SortOrder(1))
        );
    } else {
        // Note: Parse Server might not return indexes if none were *explicitly* set beyond defaults like _id.
        // If indexes_to_add was empty, this branch might be hit. For this test, we expect it.
        // However, some Parse Server versions might not return the 'indexes' field at all if it's empty after operations.
        // For this test, we are adding one, so we expect it.
        panic!("Indexes not found in updated schema response, but an index was added.");
    }

    // Fetch again to ensure persistence
    let fetched_again_schema = client
        .get_class_schema(&class_name)
        .await
        .expect("Failed to fetch schema after update and verification");
    assert_eq!(
        fetched_again_schema.fields.len(),
        updated_schema.fields.len()
    );
    assert_eq!(
        fetched_again_schema.class_level_permissions,
        updated_schema.class_level_permissions
    );
    assert_eq!(fetched_again_schema.indexes, updated_schema.indexes);

    // 4. Clean up
    client
        .delete_class_schema(&class_name, true)
        .await
        .expect("Failed to delete class schema after update test");
}

#[tokio::test]
async fn test_delete_non_empty_class_schema_fails() {
    let client = setup_client_with_master_key();
    let class_name = unique_class_name("TestNonEmptyDelete");

    // 1. Create a class schema
    let schema_payload = json!({
        "className": class_name,
        "fields": {
            "message": { "type": "String" }
        },
        "classLevelPermissions": { // Make it fully open for easy object creation/deletion
            "find": { "*": true },
            "get": { "*": true },
            "create": { "*": true },
            "update": { "*": true },
            "delete": { "*": true },
            "addField": { "*": true }
        }
    });
    client
        .create_class_schema(&class_name, &schema_payload)
        .await
        .expect("Failed to create schema for non-empty delete test");

    // 2. Create an object in that class
    let object_payload = json!({ "message": "Hello, World!" });
    let created_object = client
        .create_object(&class_name, &object_payload)
        .await
        .expect("Failed to create object in class");
    assert!(!created_object.object_id.is_empty());

    // 3. Attempt to delete the class schema (should fail as it's not empty)
    match client.delete_class_schema(&class_name, true).await {
        Err(ParseError::OtherParseError { code, message }) => {
            assert_eq!(
                code, 255,
                "Expected error code 255 for non-empty class deletion"
            );
            assert!(message.to_lowercase().contains("not empty"));
        }
        Ok(_) => panic!("Deleting non-empty schema should have failed but succeeded."),
        Err(e) => panic!(
            "Deleting non-empty schema failed with an unexpected error: {}",
            e
        ),
    }

    // 4. Clean up: Delete the object first, then the schema
    client
        .delete_object(&class_name, &created_object.object_id)
        .await
        .expect("Failed to delete object for cleanup");

    client
        .delete_class_schema(&class_name, true)
        .await
        .expect("Failed to delete class schema for cleanup after deleting object");

    // Verify schema is actually gone
    match client.get_class_schema(&class_name).await {
        Err(ParseError::OtherParseError { code, message: _ }) => {
            assert_eq!(code, 103);
        }
        Ok(_) => panic!("Schema should not exist after cleanup but was found."),
        Err(e) => panic!(
            "Fetching schema after cleanup failed with an unexpected error: {}",
            e
        ),
    }
}
