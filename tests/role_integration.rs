use parse_rs::role::NewParseRole;
use parse_rs::Parse;
use parse_rs::ParseACL;
use uuid::Uuid;

mod query_test_utils;
use query_test_utils::shared::setup_client_with_master_key;

async fn cleanup_role(client: &Parse, role_id: &str) {
    let _ = client.delete_role(role_id).await;
}

#[tokio::test]
async fn test_create_and_get_role() {
    let client = setup_client_with_master_key();
    let role_name = format!("TestRole_{}", Uuid::new_v4().simple());

    let mut acl = ParseACL::new();
    acl.set_public_read_access(true);
    acl.set_public_write_access(false);

    let new_role = NewParseRole {
        name: role_name.clone(),
        acl,
    };

    let created_role_response = client.create_role(&new_role).await;
    assert!(
        created_role_response.is_ok(),
        "Failed to create role: {:?}",
        created_role_response.err()
    );
    let created_role = created_role_response.unwrap();
    assert_eq!(created_role.name, role_name);
    assert!(
        created_role.object_id.is_some(),
        "Created role should have an objectId"
    );
    let role_object_id = created_role.object_id.unwrap();

    let retrieved_role_response = client.get_role(&role_object_id).await;
    assert!(
        retrieved_role_response.is_ok(),
        "Failed to get role: {:?}",
        retrieved_role_response.err()
    );
    let retrieved_role = retrieved_role_response.unwrap();
    assert_eq!(retrieved_role.name, role_name);
    assert_eq!(retrieved_role.object_id, Some(role_object_id.clone()));

    // Cleanup
    cleanup_role(&client, &role_object_id).await;
}
