// tests/file_integration.rs
use parse_rs::{FileField, ParseObject, RetrievedParseObject};
use serde_json::json;

mod query_test_utils;

use query_test_utils::shared::setup_client_with_master_key;

#[tokio::test]
async fn test_upload_file_and_associate_with_object() {
    let client = setup_client_with_master_key();
    let class_name = "TestFileObject";

    // 1. Create file data
    let file_name = "test_upload.txt";
    let file_content = "Hello, Parse File world!".as_bytes().to_vec();
    let mime_type = "text/plain";

    // 2. Upload the file
    let upload_response = client
        .upload_file(file_name, file_content, mime_type)
        .await
        .expect("Failed to upload file");

    assert!(
        upload_response.name.ends_with(file_name),
        "Uploaded file name '{}' should end with the original file name '{}'",
        upload_response.name,
        file_name
    );
    assert!(
        upload_response.url.starts_with(client.server_url.as_str()),
        "Uploaded file URL should start with server URL"
    );
    assert!(
        upload_response.url.contains(file_name),
        "Uploaded file URL should contain file name"
    );

    // 3. Create a ParseObject and associate the uploaded file
    let mut object_to_create = ParseObject::new(class_name);
    let file_field = FileField::new(upload_response.name.clone(), upload_response.url.clone());
    object_to_create.set("myTestFile", json!(file_field));

    let create_response = client
        .create_object(class_name, &object_to_create.fields)
        .await
        .expect("Failed to create object with file");
    let object_id = create_response.object_id;

    // 4. Retrieve the object and verify the file field
    let retrieved_object: RetrievedParseObject = client
        .retrieve_object(class_name, &object_id)
        .await
        .expect("Failed to retrieve object");

    let retrieved_file_field_val = retrieved_object
        .fields
        .get("myTestFile")
        .expect("myTestFile field missing");

    let retrieved_file_field: FileField = serde_json::from_value(retrieved_file_field_val.clone())
        .expect("Failed to deserialize FileField from retrieved object");

    assert_eq!(retrieved_file_field.name, upload_response.name);
    assert_eq!(retrieved_file_field.url, upload_response.url);
    assert_eq!(retrieved_file_field._type, "File");

    // Clean up: Delete the object
    client
        .delete_object(class_name, &object_id)
        .await
        .expect("Failed to delete object");

    // Note: Deleting the file itself from Parse Server is typically handled by Parse Server's file adapter settings
    // or manually if direct file deletion API is available and implemented.
    // For this test, we only ensure the object referencing it is deleted.
}
