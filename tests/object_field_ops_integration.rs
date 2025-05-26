// tests/object_field_ops_integration.rs
use parse_rs::{ParseObject, RetrievedParseObject};
use serde_json::json;

mod query_test_utils;
use query_test_utils::shared::setup_client_with_master_key;

// Helper function to create a unique class name for testing to avoid collisions
fn unique_class_name(base: &str) -> String {
    use uuid::Uuid;
    format!("{}_{}", base, Uuid::new_v4().simple())
}

#[tokio::test]
async fn test_increment_operation() {
    let client = setup_client_with_master_key();
    let class_name = unique_class_name("TestItemInc");
    let mut object_to_create = ParseObject::new(&class_name);
    object_to_create.set("score", 10);

    // Create the object
    let create_response = client
        .create_object(&class_name, &object_to_create.fields)
        .await
        .expect("Failed to create object");
    let object_id = create_response.object_id;
    object_to_create.object_id = Some(object_id.clone()); // Update local object with id

    // Stage increment operation
    object_to_create.increment("score", 5);

    // Save the object with the increment operation
    client
        .update_object(&class_name, &object_id, &object_to_create.fields)
        .await
        .expect("Failed to update object with increment");

    // Retrieve the object to verify
    let retrieved_object: RetrievedParseObject = client
        .retrieve_object(&class_name, &object_id)
        .await
        .expect("Failed to retrieve object");

    let score_val = retrieved_object
        .fields
        .get("score")
        .expect("Score field missing");
    assert_eq!(
        score_val.as_i64(),
        Some(15),
        "Score should be incremented to 15"
    );

    // Clean up
    client
        .delete_object(&class_name, &object_id)
        .await
        .expect("Failed to delete object");
}

#[tokio::test]
async fn test_decrement_operation() {
    let client = setup_client_with_master_key();
    let class_name = unique_class_name("TestItemDec");
    let mut object_to_create = ParseObject::new(&class_name);
    object_to_create.set("score", 20);

    let create_response = client
        .create_object(&class_name, &object_to_create.fields)
        .await
        .expect("Failed to create object");
    let object_id = create_response.object_id;
    object_to_create.object_id = Some(object_id.clone());

    object_to_create.decrement("score", 7);
    client
        .update_object(&class_name, &object_id, &object_to_create.fields)
        .await
        .expect("Failed to update object with decrement");

    let retrieved_object: RetrievedParseObject = client
        .retrieve_object(&class_name, &object_id)
        .await
        .expect("Failed to retrieve object");
    let score_val = retrieved_object
        .fields
        .get("score")
        .expect("Score field missing");
    assert_eq!(
        score_val.as_i64(),
        Some(13),
        "Score should be decremented to 13"
    );

    client
        .delete_object(&class_name, &object_id)
        .await
        .expect("Failed to delete object");
}

#[tokio::test]
async fn test_add_to_array_operation() {
    let client = setup_client_with_master_key();
    let class_name = unique_class_name("TestItemArrayAdd");
    let mut object_to_create = ParseObject::new(&class_name);
    object_to_create.set("tags", vec!["a", "b"]);

    let create_response = client
        .create_object(&class_name, &object_to_create.fields)
        .await
        .expect("Failed to create object");
    let object_id = create_response.object_id;
    object_to_create.object_id = Some(object_id.clone());

    object_to_create.add_to_array("tags", &["c", "a"]); // Add 'c' and a duplicate 'a'
    client
        .update_object(&class_name, &object_id, &object_to_create.fields)
        .await
        .expect("Failed to update object with add_to_array");

    let retrieved_object: RetrievedParseObject = client
        .retrieve_object(&class_name, &object_id)
        .await
        .expect("Failed to retrieve object");
    let tags_val = retrieved_object
        .fields
        .get("tags")
        .expect("Tags field missing")
        .as_array()
        .expect("Tags should be an array");

    // Expected: ["a", "b", "c", "a"]
    let expected_tags: Vec<serde_json::Value> =
        vec![json!("a"), json!("b"), json!("c"), json!("a")];
    assert_eq!(tags_val.len(), 4);
    assert!(tags_val.contains(&expected_tags[0]));
    assert!(tags_val.contains(&expected_tags[1]));
    assert!(tags_val.contains(&expected_tags[2]));
    // Check count of 'a'
    assert_eq!(tags_val.iter().filter(|&v| v == &json!("a")).count(), 2);

    client
        .delete_object(&class_name, &object_id)
        .await
        .expect("Failed to delete object");
}

#[tokio::test]
async fn test_add_unique_to_array_operation() {
    let client = setup_client_with_master_key();
    let class_name = unique_class_name("TestItemArrayAddUnique");
    let mut object_to_create = ParseObject::new(&class_name);
    object_to_create.set("tags", vec!["a", "b"]);

    let create_response = client
        .create_object(&class_name, &object_to_create.fields)
        .await
        .expect("Failed to create object");
    let object_id = create_response.object_id;
    object_to_create.object_id = Some(object_id.clone());

    object_to_create.add_unique_to_array("tags", &["c", "a"]); // Add 'c' and an existing 'a'
    client
        .update_object(&class_name, &object_id, &object_to_create.fields)
        .await
        .expect("Failed to update object with add_unique_to_array");

    let retrieved_object: RetrievedParseObject = client
        .retrieve_object(&class_name, &object_id)
        .await
        .expect("Failed to retrieve object");
    let tags_val = retrieved_object
        .fields
        .get("tags")
        .expect("Tags field missing")
        .as_array()
        .expect("Tags should be an array");

    // Expected: ["a", "b", "c"]
    let expected_tags: Vec<serde_json::Value> = vec![json!("a"), json!("b"), json!("c")];
    assert_eq!(tags_val.len(), 3);
    assert!(tags_val.contains(&expected_tags[0]));
    assert!(tags_val.contains(&expected_tags[1]));
    assert!(tags_val.contains(&expected_tags[2]));

    client
        .delete_object(&class_name, &object_id)
        .await
        .expect("Failed to delete object");
}

#[tokio::test]
async fn test_remove_from_array_operation() {
    let client = setup_client_with_master_key();
    let class_name = unique_class_name("TestItemArrayRemove");
    let mut object_to_create = ParseObject::new(&class_name);
    object_to_create.set("tags", vec!["a", "b", "c", "a", "d"]);

    let create_response = client
        .create_object(&class_name, &object_to_create.fields)
        .await
        .expect("Failed to create object");
    let object_id = create_response.object_id;
    object_to_create.object_id = Some(object_id.clone());

    object_to_create.remove_from_array("tags", &["a", "d"]); // Remove all 'a's and 'd'
    client
        .update_object(&class_name, &object_id, &object_to_create.fields)
        .await
        .expect("Failed to update object with remove_from_array");

    let retrieved_object: RetrievedParseObject = client
        .retrieve_object(&class_name, &object_id)
        .await
        .expect("Failed to retrieve object");
    let tags_val = retrieved_object
        .fields
        .get("tags")
        .expect("Tags field missing")
        .as_array()
        .expect("Tags should be an array");

    // Expected: ["b", "c"]
    let expected_tags: Vec<serde_json::Value> = vec![json!("b"), json!("c")];
    assert_eq!(tags_val.len(), 2);
    assert!(tags_val.contains(&expected_tags[0]));
    assert!(tags_val.contains(&expected_tags[1]));

    client
        .delete_object(&class_name, &object_id)
        .await
        .expect("Failed to delete object");
}
