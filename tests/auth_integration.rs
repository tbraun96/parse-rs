use parse_rs::Parse;
use parse_rs::user::SignupResponse;
use std::env;
use uuid::Uuid;
use dotenvy::dotenv;

// Helper function to initialize ParseClient from environment variables loaded from .env
fn setup_client() -> Parse {
    dotenv().ok();

    let app_id = env::var("PARSE_APP_ID")
        .expect("PARSE_APP_ID not set in .env or environment for integration tests");
    
    let port = env::var("PARSE_SERVER_PORT").expect("PARSE_SERVER_PORT not set in .env or environment for integration tests");
    let server_url = format!("http://localhost:{}/parse", port);

    Parse::new(&server_url, &app_id, None, None, None)
        .expect("Failed to create ParseClient")
}

// Helper to generate unique credentials for each test run
fn generate_unique_username() -> String {
    format!("testuser_{}", Uuid::new_v4().simple())
}

#[cfg(test)]
mod auth_tests {
    use super::*;

    #[tokio::test]
    async fn test_signup_new_user_success() {
        let mut client = setup_client();
        let username = generate_unique_username();
        let password = "testpassword123";
        let email = format!("{}@example.com", username);

        let signup_result = client
            .signup(&username, &password, Some(email.as_str()))
            .await;

        assert!(signup_result.is_ok(), "Signup failed: {:?}", signup_result.err());
        let signup_data = signup_result.unwrap(); // This is now SignupResponse

        // Fields in SignupResponse are not Options, they are guaranteed on success
        assert!(!signup_data.object_id.is_empty(), "User should have a non-empty objectId");
        assert!(!signup_data.session_token.is_empty(), "User should have a non-empty sessionToken");
        // created_at is also available in signup_data if needed for assertions

        // Check client's internal state (session token should be set by the signup method)
        assert!(client.is_authenticated(), "Client should be authenticated after signup");
        assert_eq!(client.get_session_token(), Some(signup_data.session_token.as_str()), "Client session token should match user's session token");
    }

    #[tokio::test]
    async fn test_user_signup_login_logout_get_current_user() {
        // Load .env file
        dotenvy::dotenv().ok();

        // Setup: Ensure Parse Server is running and accessible
        let app_id = env::var("PARSE_APP_ID").expect("PARSE_APP_ID not set in .env or environment for integration tests");
        let port = env::var("PARSE_SERVER_PORT").expect("PARSE_SERVER_PORT not set in .env or environment for integration tests");
        let server_url = format!("http://localhost:{}/parse", port);

        // Initialize Parse client
        let client = Parse::new(&server_url, &app_id, None, None, None)
            .expect("Failed to create ParseClient");
    }

    // More tests will go here: login, logout, current_user, request_password_reset etc.
}
