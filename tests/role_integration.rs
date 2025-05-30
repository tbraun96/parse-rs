use parse_rs::role::NewParseRole;
use parse_rs::Parse;
use parse_rs::ParseACL;
use uuid::Uuid;

mod query_test_utils;
use parse_rs::user::SignupRequest;
use query_test_utils::shared::setup_client_with_master_key; // Added for user creation in tests

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

#[tokio::test]
async fn test_add_remove_users_in_role() {
    let mut client = setup_client_with_master_key(); // Mutable for user().signup()

    // 1. Create a test user
    let username = format!("RoleUser_{}", Uuid::new_v4().simple());
    let password = "password123";
    let email_string = format!("{}@example.com", username);
    let signup_details = SignupRequest {
        username: &username,
        password,
        email: Some(&email_string),
    };
    let user_signup_response = client
        .user()
        .signup(&signup_details)
        .await
        .expect("Failed to create user for role test");
    let user_object_id = user_signup_response.object_id;

    // 2. Create a test role
    let role_name = format!("UserMgmtRole_{}", Uuid::new_v4().simple());
    let mut acl = ParseACL::new();
    acl.set_public_read_access(true);
    // For this test, master key is used for role modifications, so role's own ACL for write isn't strictly necessary
    // but good practice to set it if non-master key operations were intended.
    acl.set_public_write_access(false); // Or true, depending on test design if not using master key for role ops

    let new_role = NewParseRole {
        name: role_name.clone(),
        acl,
    };
    let created_role = client
        .create_role(&new_role)
        .await
        .expect("Failed to create role for user management test");
    let role_object_id = created_role.object_id.clone().unwrap();

    // 3. Add user to role
    let add_response = client
        .add_users_to_role(&role_object_id, &[&user_object_id])
        .await;
    assert!(
        add_response.is_ok(),
        "Failed to add user to role: {:?}",
        add_response.err()
    );

    // 4. Remove user from role
    let remove_response = client
        .remove_users_from_role(&role_object_id, &[&user_object_id])
        .await;
    assert!(
        remove_response.is_ok(),
        "Failed to remove user from role: {:?}",
        remove_response.err()
    );

    // 5. Cleanup
    cleanup_role(&client, &role_object_id).await;
    client
        .delete_user(&user_object_id)
        .await
        .expect("Failed to delete test user");
}

#[tokio::test]
async fn test_add_remove_child_roles_in_role() {
    let client = setup_client_with_master_key();

    // 1. Create a parent role
    let parent_role_name = format!("ParentRole_{}", Uuid::new_v4().simple());
    let mut parent_acl = ParseACL::new();
    parent_acl.set_public_read_access(true);
    let new_parent_role = NewParseRole {
        name: parent_role_name.clone(),
        acl: parent_acl,
    };
    let created_parent_role = client
        .create_role(&new_parent_role)
        .await
        .expect("Failed to create parent role");
    let parent_role_object_id = created_parent_role.object_id.clone().unwrap();

    // 2. Create a child role
    let child_role_name = format!("ChildRole_{}", Uuid::new_v4().simple());
    let mut child_acl = ParseACL::new();
    child_acl.set_public_read_access(true);
    let new_child_role = NewParseRole {
        name: child_role_name.clone(),
        acl: child_acl,
    };
    let created_child_role = client
        .create_role(&new_child_role)
        .await
        .expect("Failed to create child role");
    let child_role_object_id = created_child_role.object_id.clone().unwrap();

    // 3. Add child role to parent role's 'roles' relation
    let add_rel_response = client
        .add_child_roles_to_role(&parent_role_object_id, &[&child_role_object_id])
        .await;
    assert!(
        add_rel_response.is_ok(),
        "Failed to add child role to parent role: {:?}",
        add_rel_response.err()
    );

    // 4. Remove child role from parent role's 'roles' relation
    let remove_rel_response = client
        .remove_child_roles_from_role(&parent_role_object_id, &[&child_role_object_id])
        .await;
    assert!(
        remove_rel_response.is_ok(),
        "Failed to remove child role from parent role: {:?}",
        remove_rel_response.err()
    );

    // 5. Cleanup
    cleanup_role(&client, &child_role_object_id).await;
    cleanup_role(&client, &parent_role_object_id).await;
}
