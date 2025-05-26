use parse_rs::user::{LoginRequest, PasswordResetRequest, SignupRequest};
use parse_rs::ParseError;
use uuid::Uuid;

mod query_test_utils;

fn generate_unique_username() -> String {
    format!("testuser_{}", Uuid::new_v4().simple())
}

#[cfg(test)]
mod auth_tests {
    use super::query_test_utils::shared::setup_client;
    use super::*;

    #[tokio::test]
    async fn test_signup_new_user_success() {
        let mut client = setup_client();
        let username_str = generate_unique_username();
        let username = username_str.as_str();
        let password = "testpassword123";
        let email_str = format!("{}@example.com", username);
        let email = Some(email_str.as_str());

        let signup_request = SignupRequest {
            username,
            password,
            email,
        };
        let signup_result = client.user().signup(&signup_request).await;

        assert!(
            signup_result.is_ok(),
            "Signup failed: {:?}",
            signup_result.err()
        );
        let signup_data = signup_result.unwrap(); // This is now SignupResponse

        // Fields in SignupResponse are not Options, they are guaranteed on success
        assert!(
            !signup_data.object_id.is_empty(),
            "User should have a non-empty objectId"
        );
        assert!(
            !signup_data.session_token.is_empty(),
            "User should have a non-empty sessionToken"
        );
        // created_at is also available in signup_data if needed for assertions

        // Check client's internal state (session token should be set by the signup method)
        assert!(
            client.is_authenticated(),
            "Client should be authenticated after signup"
        );
        assert_eq!(
            client.session_token(),
            Some(signup_data.session_token.as_str()),
            "Client session token should match user's session token"
        );
    }

    #[tokio::test]
    async fn test_user_signup_login_logout_get_current_user() {
        let mut client = setup_client(); // client needs to be mutable for signup, logout, login
        let username_str = generate_unique_username();
        let username = username_str.as_str();
        let password = "testpassword123";
        let email_str = format!("{}@example.com", username);
        let email = Some(email_str.as_str());

        // 1. Signup
        let signup_request = SignupRequest {
            username,
            password,
            email,
        };
        let signup_result = client.user().signup(&signup_request).await;
        assert!(
            signup_result.is_ok(),
            "Signup failed: {:?}",
            signup_result.err()
        );
        let signup_data = signup_result.unwrap();
        let original_object_id = signup_data.object_id.clone();
        let signup_session_token = signup_data.session_token.clone();

        assert!(
            client.is_authenticated(),
            "Client should be authenticated after signup"
        );
        assert_eq!(
            client.session_token(),
            Some(signup_session_token.as_str()),
            "Client session token should match signup session token"
        );

        // 2. Get Current User (after signup)
        let current_user_after_signup_result = client.user().me().await;
        assert!(
            current_user_after_signup_result.is_ok(),
            "current_user after signup failed: {:?}",
            current_user_after_signup_result.err()
        );
        let current_user_after_signup = current_user_after_signup_result.unwrap();
        let user1 = current_user_after_signup.clone();
        assert_eq!(
            user1.object_id.as_deref(),
            Some(original_object_id.as_str()),
            "ObjectID mismatch after signup"
        );
        assert_eq!(user1.username, username, "Username mismatch after signup");
        // The /users/me endpoint DOES return the session token for the current user.
        // This token should match the one obtained during signup.
        assert_eq!(
            user1.session_token.as_deref(),
            Some(signup_session_token.as_str()),
            "Session token from /users/me should match signup session token"
        );

        // 3. Logout
        let logout_result = client.user().logout().await;
        assert!(
            logout_result.is_ok(),
            "Logout failed: {:?}",
            logout_result.err()
        );
        assert!(
            !client.is_authenticated(),
            "Client should not be authenticated after logout"
        );

        // 4. Get Current User (after logout)
        let current_user_after_logout_result = client.user().me().await;
        assert!(
            current_user_after_logout_result.is_err(),
            "current_user after logout should fail"
        );

        // 5. Login
        let login_request = LoginRequest { username, password };
        let login_result = client.user().login(&login_request).await;
        assert!(
            login_result.is_ok(),
            "Login failed: {:?}",
            login_result.err()
        );
        let logged_in_user = login_result.unwrap();
        assert_eq!(
            logged_in_user.object_id.as_deref(),
            Some(original_object_id.as_str()),
            "ObjectID mismatch after login"
        );
        assert_eq!(
            logged_in_user.username, username,
            "Username mismatch after login"
        );
        assert!(
            logged_in_user.session_token.is_some(),
            "Login response should contain sessionToken"
        );
        let login_session_token = logged_in_user.session_token.unwrap();

        assert!(
            client.is_authenticated(),
            "Client should be authenticated after login"
        );
        assert_eq!(
            client.session_token(),
            Some(login_session_token.as_str()),
            "Client session token should match login session token"
        );

        // 6. Get Current User (after login)
        let current_user_after_login_result = client.user().me().await;
        assert!(
            current_user_after_login_result.is_ok(),
            "current_user after login failed: {:?}",
            current_user_after_login_result.err()
        );
        let current_user_after_login = current_user_after_login_result.unwrap();
        let user2 = current_user_after_login.clone();
        assert_eq!(
            user2.object_id.as_deref(),
            Some(original_object_id.as_str()),
            "ObjectID mismatch after login (current_user)"
        );
        assert_eq!(
            user2.username, username,
            "Username mismatch after login (current_user)"
        );
        // The /users/me endpoint returns the session token.
        // This token should match the one obtained during the most recent login.
        assert_eq!(
            user2.session_token.as_deref(),
            Some(login_session_token.as_str()),
            "Session token from /users/me should match current login session token"
        );

        // 7. Logout (final)
        let logout_result2 = client.user().logout().await;
        assert!(
            logout_result2.is_ok(),
            "Second logout failed: {:?}",
            logout_result2.err()
        );
        assert!(
            !client.is_authenticated(),
            "Client should not be authenticated after final logout"
        );
    }

    #[tokio::test]
    async fn test_login_invalid_credentials() {
        let mut client = setup_client();
        let username = "nonexistentuser";
        let password = "wrongpassword";

        let login_request = LoginRequest { username, password };
        let login_result = client.user().login(&login_request).await;

        if let Err(ParseError::ObjectNotFound(error_message)) = login_result {
            assert!(
                error_message.contains("(101)"),
                "Error message for invalid login should contain (101). Got: {}",
                error_message
            );
        } else {
            panic!(
                "Expected ParseError::ObjectNotFound for invalid login, got: {:?}",
                login_result
            );
        }

        assert!(
            !client.is_authenticated(),
            "Client should not be authenticated after failed login"
        );
    }

    #[tokio::test]
    async fn test_signup_with_existing_credentials() {
        let mut client = setup_client();
        let original_username_str = generate_unique_username();
        let original_username = original_username_str.as_str();
        let original_password = "password123";
        let original_email_str = format!("{}@example.com", original_username);
        let original_email = Some(original_email_str.as_str());

        // First signup
        let signup_request_1 = SignupRequest {
            username: original_username,
            password: original_password,
            email: original_email,
        };
        let signup_result1 = client.user().signup(&signup_request_1).await;
        assert!(
            signup_result1.is_ok(),
            "Initial signup failed: {:?}",
            signup_result1.err()
        );
        client
            .user()
            .logout()
            .await
            .expect("Logout after initial signup failed"); // Logout to ensure clean state for next signup attempt

        // Attempt signup with same username
        let new_password_same_username = "newpass1";
        let new_email_same_username_str = format!("new_{}", original_email_str);
        let new_email_same_username = Some(new_email_same_username_str.as_str());

        let signup_request_2 = SignupRequest {
            username: original_username,
            password: new_password_same_username,
            email: new_email_same_username,
        };
        let signup_result2 = client.user().signup(&signup_request_2).await;
        assert!(
            signup_result2.is_err(),
            "Signup with existing username should fail"
        );
        if let Err(ParseError::UsernameTaken(error_message)) = signup_result2 {
            assert!(
                error_message.contains("(202)"),
                "Expected error message for existing username to contain (202). Got: {}",
                error_message
            );
        } else {
            panic!(
                "Expected ParseError::UsernameTaken for existing username, got: {:?}",
                signup_result2
            );
        }

        // Attempt signup with same email
        let new_username_same_email_str = generate_unique_username();
        let new_username_same_email = new_username_same_email_str.as_str();
        let new_password_same_email = "newpass2";

        let signup_request_3 = SignupRequest {
            username: new_username_same_email,
            password: new_password_same_email,
            email: original_email,
        };
        let signup_result3 = client.user().signup(&signup_request_3).await;
        assert!(
            signup_result3.is_err(),
            "Signup with existing email should fail"
        );
        if let Err(parse_rs::ParseError::EmailTaken(error_message)) = signup_result3 {
            assert!(
                error_message.contains("(203)"),
                "Expected error message for existing email to contain (203). Got: {}",
                error_message
            );
        } else {
            panic!(
                "Expected ApiError for existing email, got: {:?}",
                signup_result3
            );
        }
    }

    #[tokio::test]
    async fn test_request_password_reset() {
        let mut client = setup_client();
        let username_str = generate_unique_username();
        let username = username_str.as_str();
        let password = "resetpass123";
        let email_str = format!("{}@example.com", username);
        let email = email_str.as_str(); // for PasswordResetRequest
        let signup_email = Some(email_str.as_str()); // for SignupRequest

        // First, create a user
        let signup_request = SignupRequest {
            username,
            password,
            email: signup_email,
        };
        let signup_result = client.user().signup(&signup_request).await;
        assert!(
            signup_result.is_ok(),
            "Signup failed: {:?}",
            signup_result.err()
        );

        // Logout if client was authenticated by signup
        if client.is_authenticated() {
            client
                .user()
                .logout()
                .await
                .expect("Logout failed before password reset test");
        }

        // Request password reset for the user's email
        let reset_request = PasswordResetRequest { email };
        let reset_result = client.user().request_password_reset(&reset_request).await;

        if let Err(ParseError::InternalServerError(error_message)) = reset_result {
            assert!(
                error_message.contains("(1)") && error_message.to_lowercase().contains("emailadapter are required"),
                "Error message for password reset config should contain (1) and mention emailAdapter. Got: {}",
                error_message
            );
        } else {
            panic!(
                "Expected ParseError::InternalServerError for password reset due to server config, got: {:?}",
                reset_result
            );
        }

        // Test with a non-existent email
        // Since the server email is not configured, we still expect error code 1,
        // as the server won't proceed to check if the email exists.
        let non_existent_email_str = format!("nonexistent_{}", email_str);
        let non_existent_email = non_existent_email_str.as_str();
        let reset_request_non_existent = PasswordResetRequest {
            email: non_existent_email,
        };
        let reset_result_non_existent = client
            .user()
            .request_password_reset(&reset_request_non_existent)
            .await;

        // Expect InternalServerError due to server not being configured for email,
        // even for a non-existent email.
        if let Err(ParseError::InternalServerError(error_message)) = reset_result_non_existent {
            assert!(
                error_message.contains("(1)") && error_message.to_lowercase().contains("emailadapter are required"),
                "Error for non-existent email (due to server config) should contain (1) and mention emailAdapter. Got: {}",
                error_message
            );
        } else {
            panic!(
                "Expected ParseError::InternalServerError for non-existent email due to server config, got: {:?}",
                reset_result_non_existent
            );
        }
    }

    #[tokio::test]
    async fn test_user_become_another_user() {
        // 1. Setup User A (the user to be "become")
        let mut client_a = setup_client();
        let user_a_username_str = generate_unique_username();
        let user_a_username = user_a_username_str.as_str();
        let user_a_email_str = format!("{}@example.com", user_a_username);
        let user_a_email = Some(user_a_email_str.as_str());
        let user_a_password = "passwordA";

        let signup_a_req = SignupRequest {
            username: user_a_username,
            password: user_a_password,
            email: user_a_email,
        };
        let signup_a_res = client_a
            .user()
            .signup(&signup_a_req)
            .await
            .expect("User A signup failed");
        let user_a_object_id = signup_a_res.object_id.clone();
        let user_a_session_token = signup_a_res.session_token.clone();
        // DO NOT log out User A here. Their session must be active for "become" to work with their token.

        // 2. Setup User B (the user who will initiate "become")
        let mut client_b = setup_client();
        let user_b_username_str = generate_unique_username();
        let user_b_username = user_b_username_str.as_str();
        let user_b_email_str = format!("{}@example.com", user_b_username);
        let user_b_email = Some(user_b_email_str.as_str());
        let user_b_password = "passwordB";

        let signup_b_req = SignupRequest {
            username: user_b_username,
            password: user_b_password,
            email: user_b_email,
        };
        let _signup_b_res = client_b
            .user()
            .signup(&signup_b_req)
            .await
            .expect("User B signup failed");
        // client_b is now authenticated as User B.

        // 3. User B attempts to "become" User A
        let become_result = client_b.user().become_user(&user_a_session_token).await;
        assert!(
            become_result.is_ok(),
            "Become call failed: {:?}",
            become_result.err()
        );

        let became_user_details = become_result.unwrap();
        assert_eq!(
            became_user_details.object_id.as_deref(),
            Some(user_a_object_id.as_str()),
            "Become: ObjectID mismatch"
        );
        assert_eq!(
            became_user_details.username, user_a_username,
            "Become: Username mismatch"
        );
        assert_eq!(
            became_user_details.session_token.as_deref(),
            Some(user_a_session_token.as_str()),
            "Become: Session token in response mismatch"
        );

        // 4. Verify client_b's internal session token is now User A's
        assert_eq!(
            client_b.session_token(),
            Some(user_a_session_token.as_str()),
            "Client's internal session token did not update after become"
        );

        // 5. Verify current_user for client_b is now User A
        let current_user_after_become = client_b
            .user()
            .me()
            .await
            .expect("current_user after become failed");
        assert_eq!(
            current_user_after_become.object_id.as_deref(),
            Some(user_a_object_id.as_str()),
            "current_user after become: ObjectID mismatch"
        );
        assert_eq!(
            current_user_after_become.username, user_a_username,
            "current_user after become: Username mismatch"
        );
        assert_eq!(
            current_user_after_become.session_token.as_deref(),
            Some(user_a_session_token.as_str()),
            "current_user after become: Session token mismatch"
        );

        // 6. Clean up: Logout (which will use User A's session token now stored in client_b)
        client_b
            .user()
            .logout()
            .await
            .expect("Logout after become (as User A) failed");
        assert!(
            !client_b.is_authenticated(),
            "Client should not be authenticated after final logout"
        );
    }
}
