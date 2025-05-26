// tests/acl_integration.rs

use chrono::Utc;
use parse_rs::acl::ParseACL;
use parse_rs::object::ParseObject;
use serde_json::{json, Value};

use crate::query_test_utils::shared::setup_client_with_master_key;

// Assuming query_test_utils is at the root of the tests directory or accessible via crate path
// If query_test_utils is in the same directory, you might need a mod.rs in tests/
// or adjust the path. For now, let's assume it's accessible.
mod query_test_utils;

#[tokio::test]
async fn test_set_and_retrieve_public_read_acl() {
    let client = setup_client_with_master_key();
    let class_name = "ACLTestObjectPublicRead";
    let mut object_ids_to_cleanup: Vec<String> = Vec::new();

    let mut acl = ParseACL::new();
    acl.set_public_read_access(true);
    acl.set_public_write_access(false); // Explicitly set for clarity

    let mut obj_to_create = ParseObject::new(class_name);
    obj_to_create.set("name", "ObjectWithPublicRead");
    obj_to_create.set_acl(acl.clone());

    match client.create_object(class_name, &obj_to_create).await {
        Ok(response) => {
            object_ids_to_cleanup.push(response.object_id.clone());
            match client
                .retrieve_object(class_name, &response.object_id)
                .await
            {
                Ok(retrieved_obj) => {
                    assert!(
                        retrieved_obj.acl.is_some(),
                        "ACL should be present on retrieved object"
                    );
                    let retrieved_acl = retrieved_obj.acl.unwrap();

                    // Serialize both ACLs to JSON for a comparable format,
                    // as direct comparison of ParseACL might be tricky if internal order differs
                    // or if the server adds/modifies default permissions.
                    let _expected_acl_json = serde_json::to_value(acl).unwrap();
                    let retrieved_acl_json = serde_json::to_value(retrieved_acl).unwrap();

                    // Parse server might add default master key access to ACLs if master key is used for creation.
                    // For this test, we primarily care that public read is true and public write is false.
                    // A more robust check would be to inspect the specific permissions.
                    let public_perms = retrieved_acl_json.get("*").unwrap().as_object().unwrap();
                    assert_eq!(public_perms.get("read"), Some(&Value::Bool(true)));
                    assert_eq!(public_perms.get("write"), None); // Can't write
                }
                Err(e) => panic!("Failed to retrieve object with ACL: {:?}", e),
            }
        }
        Err(e) => panic!("Failed to create object with ACL: {:?}", e),
    }

    // Cleanup
    for object_id in object_ids_to_cleanup {
        let endpoint = format!("classes/{}/{}", class_name, object_id);
        // Delete the object using master key to bypass ACLs for cleanup
        match client.delete_object_with_master_key(&endpoint).await {
            Ok(_) => (),
            Err(e) => panic!("Failed to cleanup object {}: {:?}", object_id, e),
        }
    }
}

#[tokio::test]
async fn test_set_and_retrieve_user_specific_acl() {
    let mut client = setup_client_with_master_key(); // Master key needed to create users for testing
    let class_name = "ACLTestObjectUserSpecific";
    let mut object_ids_to_cleanup: Vec<String> = Vec::new();
    let mut user_ids_to_cleanup: Vec<String> = Vec::new();

    // 1. Create a test user
    let username = format!("testuser_{}", Utc::now().timestamp_micros());
    let password = "testpassword";
    let user_data = json!({
        "username": username,
        "password": password,
        "email": format!("{}@example.com", username)
    });
    let user_signup_response = client
        .user()
        .signup(&user_data)
        .await
        .expect("Failed to create test user");
    let user_id = user_signup_response.object_id;
    user_ids_to_cleanup.push(user_id.clone());

    // 2. Create ACL for this user
    let mut acl = ParseACL::new();
    acl.set_user_read_access(&user_id, true);
    acl.set_user_write_access(&user_id, false);
    // By default, public access should be false
    acl.set_public_read_access(false);
    acl.set_public_write_access(false);

    // 3. Create object with this ACL
    let mut obj_to_create = ParseObject::new(class_name);
    obj_to_create.set("owner", &user_id);
    obj_to_create.set_acl(acl.clone());

    match client.create_object(class_name, &obj_to_create).await {
        Ok(response) => {
            object_ids_to_cleanup.push(response.object_id.clone());
            match client
                .retrieve_object(class_name, &response.object_id)
                .await
            {
                Ok(retrieved_obj) => {
                    assert!(retrieved_obj.acl.is_some(), "ACL should be present");
                    let retrieved_acl = retrieved_obj.acl.unwrap();
                    let retrieved_acl_json = serde_json::to_value(retrieved_acl).unwrap();

                    // Check user-specific permissions
                    let user_perms_key = user_id.to_string();
                    assert!(
                        retrieved_acl_json.get(&user_perms_key).is_some(),
                        "User permissions not found in ACL"
                    );
                    let user_perms = retrieved_acl_json
                        .get(&user_perms_key)
                        .unwrap()
                        .as_object()
                        .unwrap();
                    assert_eq!(user_perms.get("read"), Some(&Value::Bool(true)));
                    assert_eq!(user_perms.get("write"), None);

                    // Check public permissions are false
                    assert!(retrieved_acl_json.get("*").is_none());
                }
                Err(e) => panic!("Failed to retrieve object with user ACL: {:?}", e),
            }
        }
        Err(e) => panic!("Failed to create object with user ACL: {:?}", e),
    }

    // Cleanup
    for object_id in object_ids_to_cleanup {
        let endpoint = format!("classes/{}/{}", class_name, object_id);
        // Delete the object using master key to bypass ACLs for cleanup
        match client.delete_object_with_master_key(&endpoint).await {
            Ok(_) => (),
            Err(e) => panic!("Failed to cleanup object {}: {:?}", object_id, e),
        }
    }
    for user_id_to_delete in user_ids_to_cleanup {
        // Need to use master key to delete users directly by ID
        let endpoint = format!("users/{}", user_id_to_delete);
        // Use the new method from ParseClient
        match client.delete_object_with_master_key(&endpoint).await {
            Ok(_) => println!("Successfully cleaned up user: {}", user_id_to_delete),
            Err(e) => eprintln!(
                "Failed to cleanup user {}: {:?}. This might be okay if already deleted.",
                user_id_to_delete, e
            ),
        }
    }
}
