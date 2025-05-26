use chrono::Utc;
use parse_rs::ParseError;
use parse_rs::ParseSession;
use serde_json::json;
use uuid::Uuid;

// Declare query_test_utils as a module
#[path = "query_test_utils.rs"]
mod query_test_utils;
use query_test_utils::shared::{setup_client, setup_client_with_master_key};

#[tokio::test]
async fn test_session_me_success() {
    let mut client = setup_client();

    // Generate a unique username and password for signup to avoid conflicts
    let username = format!("testuser_{}", Uuid::new_v4().simple());
    let password = "testpassword123".to_string();

    let user_data = json!({
        "username": username,
        "password": password,
        "email": format!("{}@example.com", username)
    });

    // Signup a new user to get a session token
    let signup_result = client.user().signup(&user_data).await;
    assert!(
        signup_result.is_ok(),
        "Signup failed: {:?}",
        signup_result.err()
    );
    let signup_response = signup_result.unwrap();
    assert!(!signup_response.session_token.is_empty());

    // The client should now have the session token set internally by the signup method

    // Call session().me()
    let session_me_result = client.session().me().await;
    assert!(
        session_me_result.is_ok(),
        "session().me() failed: {:?}",
        session_me_result.err()
    );

    let parse_session = session_me_result.unwrap();
    assert!(
        !parse_session.session_token.is_empty(),
        "Session token in ParseSession should not be empty"
    );
    assert_eq!(
        parse_session.session_token, signup_response.session_token,
        "Session token from me() should match signup response"
    );
    let user_val = parse_session
        .user
        .as_object()
        .expect("User should be an object when included");
    assert_eq!(
        user_val.get("className").unwrap().as_str().unwrap(),
        "_User"
    );
    assert!(!user_val
        .get("objectId")
        .unwrap()
        .as_str()
        .unwrap()
        .is_empty());

    // TODO: Add cleanup - delete the created user and session if necessary
    // For now, we assume the test Parse server can handle these transient users.
}

#[tokio::test]
async fn test_session_me_fail_no_token() {
    let client = setup_client(); // A client without a session token

    let session_me_result = client.session().me().await;
    assert!(
        session_me_result.is_err(),
        "session().me() should fail without a session token"
    );

    match session_me_result.err().unwrap() {
        ParseError::SessionTokenMissing => { /* Expected error */ }
        e => panic!("Unexpected error type: {:?}", e),
    }
}

#[tokio::test]
async fn test_session_get_by_object_id_success() {
    let mut client_user_session = setup_client(); // Client for user signup and getting initial session
    let client_master_key = setup_client_with_master_key(); // Client with master key for get_by_object_id

    // 1. Signup a new user
    let username = format!("testuser_get_{}", Uuid::new_v4().simple());
    let password = "testpassword123".to_string();
    let user_data = json!({
        "username": username,
        "password": password,
    });

    let signup_result = client_user_session.user().signup(&user_data).await;
    assert!(
        signup_result.is_ok(),
        "Signup failed: {:?}",
        signup_result.err()
    );
    let signup_response = signup_result.unwrap();

    // 2. Get the current session details to find its objectId
    let current_session_result = client_user_session.session().me().await;
    assert!(
        current_session_result.is_ok(),
        "session().me() failed: {:?}",
        current_session_result.err()
    );
    let current_session = current_session_result.unwrap();
    let session_object_id = current_session.object_id.clone();
    let original_session_token = current_session.session_token.clone();

    // 3. Call get_by_object_id using the master key client
    let get_session_result = client_master_key
        .session()
        .get_by_object_id(&session_object_id)
        .await;
    assert!(
        get_session_result.is_ok(),
        "get_by_object_id failed: {:?}",
        get_session_result.err()
    );

    let retrieved_session = get_session_result.unwrap();

    // 4. Assertions
    assert_eq!(
        retrieved_session.object_id, session_object_id,
        "Retrieved session objectId does not match"
    );
    assert_eq!(
        retrieved_session.session_token, original_session_token,
        "Retrieved session token does not match"
    );
    let user_ptr_val = retrieved_session
        .user
        .as_object()
        .expect("User pointer should be an object");
    assert_eq!(
        user_ptr_val.get("className").unwrap().as_str().unwrap(),
        "_User"
    );
    assert_eq!(
        user_ptr_val.get("objectId").unwrap().as_str().unwrap(),
        signup_response.object_id
    );

    // TODO: Add cleanup - delete the created user and session if necessary
}

#[tokio::test]
async fn test_session_update_by_object_id_success() {
    let mut client_user_session = setup_client();
    let client_master_key = setup_client_with_master_key();

    // 1. Signup a new user
    let username = format!("testuser_update_{}", Uuid::new_v4().simple());
    let password = "testpassword123".to_string();
    let user_data = json!({
        "username": username,
        "password": password,
    });

    let signup_result = client_user_session.user().signup(&user_data).await;
    assert!(
        signup_result.is_ok(),
        "Signup failed: {:?}",
        signup_result.err()
    );

    // 2. Get current session's objectId
    let current_session_result = client_user_session.session().me().await;
    assert!(
        current_session_result.is_ok(),
        "session().me() failed: {:?}",
        current_session_result.err()
    );
    let current_session = current_session_result.unwrap();
    let session_object_id = current_session.object_id.clone();

    // 3. Update the session (e.g., set 'restricted' to true)
    let update_payload = json!({ "restricted": true });
    let update_result = client_master_key
        .session()
        .update_by_object_id(&session_object_id, &update_payload)
        .await;
    assert!(
        update_result.is_ok(),
        "update_by_object_id failed: {:?}",
        update_result.err()
    );
    let update_response = update_result.unwrap();
    assert!(
        !update_response.updated_at.is_empty(),
        "updatedAt should not be empty in response"
    );

    // 4. Verify the update by fetching the session again
    let get_session_result = client_master_key
        .session()
        .get_by_object_id(&session_object_id)
        .await;
    assert!(
        get_session_result.is_ok(),
        "get_by_object_id after update failed: {:?}",
        get_session_result.err()
    );
    let updated_session = get_session_result.unwrap();

    assert_eq!(updated_session.object_id, session_object_id);
    assert_eq!(
        updated_session.restricted,
        Some(true),
        "Session 'restricted' field was not updated"
    );
    assert!(
        updated_session.updated_at.is_some(),
        "updatedAt should be present in fetched session"
    );
    // We can also check if updated_session.updated_at >= update_response.updated_at if we parse them to datetimes

    // TODO: Add cleanup
}

#[tokio::test]
async fn test_session_delete_by_object_id_success() {
    let mut client_user_session = setup_client();
    let client_master_key = setup_client_with_master_key();

    // 1. Signup a new user
    let username = format!("testuser_delete_{}", Uuid::new_v4().simple());
    let password = "testpassword123".to_string();
    let user_data = json!({
        "username": username,
        "password": password,
    });

    let signup_result = client_user_session.user().signup(&user_data).await;
    assert!(
        signup_result.is_ok(),
        "Signup failed: {:?}",
        signup_result.err()
    );

    // 2. Get current session's objectId
    let current_session_result = client_user_session.session().me().await;
    assert!(
        current_session_result.is_ok(),
        "session().me() failed: {:?}",
        current_session_result.err()
    );
    let current_session = current_session_result.unwrap();
    let session_object_id = current_session.object_id.clone();

    // 3. Delete the session
    let delete_result = client_master_key
        .session()
        .delete_by_object_id(&session_object_id)
        .await;
    assert!(
        delete_result.is_ok(),
        "delete_by_object_id failed: {:?}",
        delete_result.err()
    );

    // 4. Verify the session is deleted by trying to fetch it
    let get_deleted_session_result = client_master_key
        .session()
        .get_by_object_id(&session_object_id)
        .await;
    match get_deleted_session_result {
        Err(ParseError::ObjectNotFound { .. }) => { /* Expected error */ }
        Err(ParseError::OtherParseError { code, message }) => {
            panic!(
                "Expected ObjectNotFound, but got OtherParseError: code {}, error {}",
                code, message
            );
        }
        Ok(_) => {
            panic!("Expected ObjectNotFound error, but got session successfully after deletion.")
        }
        Err(e) => panic!(
            "Expected ObjectNotFound error, but got different error: {:?}",
            e
        ),
    }
    // TODO: Add cleanup for the user if necessary, though the session is gone.
}

#[tokio::test]
async fn test_session_get_all_sessions_success() {
    let client = setup_client_with_master_key();
    let mut user_client = setup_client(); // For creating users, needs to be mut for signup

    // Create two users and their sessions
    let username1 = format!("testuser_sessions_1_{}", Utc::now().timestamp_micros());
    let password = "testpassword";
    let user1_signup_response = user_client
        .user()
        .signup(&json!({
            "username": username1,
            "password": password
        }))
        .await
        .expect("User1 signup failed");
    let session_token1 = user1_signup_response.session_token.clone();
    let user1_object_id = user1_signup_response.object_id.clone();

    let username2 = format!("testuser_sessions_2_{}", Utc::now().timestamp_micros());
    let user2_signup_response = user_client
        .user()
        .signup(&json!({
            "username": username2,
            "password": password
        }))
        .await
        .expect("User2 signup failed");
    let session_token2 = user2_signup_response.session_token.clone();
    let user2_object_id = user2_signup_response.object_id.clone();

    // Wait a bit to ensure sessions are registered on the server
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    // --- BEGIN DIAGNOSTIC QUERIES ---
    println!(
        "Attempting to get session for user1 ({}) with include=user...",
        user1_object_id
    );
    let query_user1 = format!(
        "where={{\"user\":{{\"__type\":\"Pointer\",\"className\":\"_User\",\"objectId\":\"{}\"}}}}&include=user",
        user1_object_id
    );
    let session_user1_result: Result<Vec<ParseSession>, ParseError> =
        client.session().get_all_sessions(Some(&query_user1)).await;
    if let Err(e) = &session_user1_result {
        eprintln!("Error getting session for user1 (include=user): {:?}", e);
    } else if let Ok(sessions) = &session_user1_result {
        println!(
            "Successfully retrieved {} sessions for user1 (include=user).",
            sessions.len()
        );
        if sessions.is_empty() {
            println!(
                "Warning: No session found for user1 with query: {}",
                query_user1
            );
        }
    }

    println!(
        "Attempting to get session for user2 ({}) with include=user...",
        user2_object_id
    );
    let query_user2 = format!(
        "where={{\"user\":{{\"__type\":\"Pointer\",\"className\":\"_User\",\"objectId\":\"{}\"}}}}&include=user",
        user2_object_id
    );
    let session_user2_result: Result<Vec<ParseSession>, ParseError> =
        client.session().get_all_sessions(Some(&query_user2)).await;
    if let Err(e) = &session_user2_result {
        eprintln!("Error getting session for user2 (include=user): {:?}", e);
    } else if let Ok(sessions) = &session_user2_result {
        println!(
            "Successfully retrieved {} sessions for user2 (include=user).",
            sessions.len()
        );
        if sessions.is_empty() {
            println!(
                "Warning: No session found for user2 with query: {}",
                query_user2
            );
        }
    }
    // --- END DIAGNOSTIC QUERIES ---

    // Use master key to get all sessions (first without include)
    println!("Attempting to get all sessions (no include)...");
    let sessions_no_include_result = client.session().get_all_sessions(None).await;

    if let Err(e) = &sessions_no_include_result {
        eprintln!("Error getting all sessions (no include): {:?}", e);
    }
    let sessions_no_include =
        sessions_no_include_result.expect("Failed to get sessions without user included");
    println!(
        "Successfully retrieved {} sessions (no include).",
        sessions_no_include.len()
    );
    assert!(
        sessions_no_include.len() >= 2, // We created 2 users, so there should be at least 2 sessions
        "Expected at least 2 sessions, got {}",
        sessions_no_include.len()
    );
    for session in &sessions_no_include {
        assert!(
            !session.session_token.is_empty(),
            "Session token should not be empty"
        );
        // When user is not included, it should be a pointer
        let user_ptr = session
            .user
            .as_object()
            .expect("User should be a pointer object when not included");
        assert_eq!(user_ptr.get("__type").unwrap().as_str().unwrap(), "Pointer");
        assert_eq!(
            user_ptr.get("className").unwrap().as_str().unwrap(),
            "_User"
        );
        assert!(user_ptr.contains_key("objectId"));
    }

    // Now try with include=user for our specific users
    println!("Attempting to get sessions for specific users (user1, user2) with include=user...");
    let specific_query_include_user = format!(
        "where={{\"user\":{{\"$in\":[{{\"__type\":\"Pointer\",\"className\":\"_User\",\"objectId\":\"{}\"}},{{\"__type\":\"Pointer\",\"className\":\"_User\",\"objectId\":\"{}\"}}]}}}}&include=user",
        user1_object_id,
        user2_object_id
    );
    let sessions_with_user_result = client
        .session()
        .get_all_sessions(Some(&specific_query_include_user))
        .await;

    if let Err(e) = &sessions_with_user_result {
        eprintln!(
            "Error getting sessions for specific users (user1, user2) with include=user: {:?}",
            e
        );
    }
    let sessions_with_user =
        sessions_with_user_result.expect("Failed to get sessions with user included");

    println!(
        "Successfully retrieved {} sessions for specific users (user1, user2) with include=user.",
        sessions_with_user.len()
    );
    assert!(
        !sessions_with_user.is_empty(),
        "Expected sessions results to not be empty"
    );
    // Assert that the results are not empty
    assert!(
        !sessions_with_user.is_empty(),
        "Expected sessions results to not be empty"
    );

    // Check if user1's session is present and has user details
    if let Some(found_session1) = sessions_with_user
        .iter()
        .find(|s| s.session_token == session_token1)
    {
        let user1_obj = found_session1
            .user
            .as_object()
            .expect("User1 should be an object");
        assert_eq!(
            user1_obj.get("objectId").unwrap().as_str().unwrap(),
            user1_object_id,
            "User objectId mismatch for first session"
        );
        assert_eq!(
            user1_obj.get("className").unwrap().as_str().unwrap(),
            "_User",
            "User className mismatch for first session"
        );
        assert_eq!(
            user1_obj.get("username").unwrap().as_str().unwrap(),
            username1,
            "Username mismatch for first session"
        );
    } else {
        panic!("Session for user1 not found in get_all_sessions response");
    }

    // Check if user2's session is present and has user details
    if let Some(found_session2) = sessions_with_user
        .iter()
        .find(|s| s.session_token == session_token2)
    {
        let user2_obj = found_session2
            .user
            .as_object()
            .expect("User2 should be an object");
        assert_eq!(
            user2_obj.get("objectId").unwrap().as_str().unwrap(),
            user2_object_id,
            "User objectId mismatch for second session"
        );
        assert_eq!(
            user2_obj.get("className").unwrap().as_str().unwrap(),
            "_User",
            "User className mismatch for second session"
        );
        assert_eq!(
            user2_obj.get("username").unwrap().as_str().unwrap(),
            username2,
            "Username mismatch for second session"
        );
    } else {
        panic!("Session for user2 not found in get_all_sessions response");
    }

    // Cleanup: Delete the created users (sessions are implicitly deleted with users)
    // User deletion requires master key and is a DELETE to /users/:objectId
    client
        .delete_object_with_master_key(&format!("users/{}", user1_object_id))
        .await
        .expect("Failed to delete user1");
    client
        .delete_object_with_master_key(&format!("users/{}", user2_object_id))
        .await
        .expect("Failed to delete user2");
}
