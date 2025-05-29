// src/user.rs

use crate::object::{deserialize_string_to_option_parse_date, deserialize_string_to_parse_date};
use crate::types::ParseDate;
use crate::ParseError;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents a Parse Server User object.
///
/// This struct contains standard fields for a user, such as `objectId`, `username`, `email`,
/// `emailVerified`, `sessionToken`, `createdAt`, and `updatedAt`.
/// It is used to deserialize user data received from the Parse Server and can also be
/// (though less commonly for `ParseUser` itself) used for creating or updating user objects.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParseUser {
    #[serde(rename = "objectId")]
    pub object_id: Option<String>,
    pub username: String,
    pub email: Option<String>,
    #[serde(rename = "emailVerified")]
    pub email_verified: Option<bool>,
    #[serde(rename = "sessionToken")]
    pub session_token: Option<String>,
    #[serde(
        rename = "createdAt",
        deserialize_with = "deserialize_string_to_option_parse_date",
        skip_serializing_if = "Option::is_none"
    )]
    pub created_at: Option<ParseDate>,
    #[serde(
        rename = "updatedAt",
        deserialize_with = "deserialize_string_to_option_parse_date",
        skip_serializing_if = "Option::is_none"
    )]
    pub updated_at: Option<ParseDate>,
}

// New struct for signup response
/// Represents the successful response from a user signup operation.
///
/// It includes the `objectId` of the newly created user, their `sessionToken`,
/// and the `createdAt` timestamp.
#[derive(Debug, Deserialize, Clone)]
pub struct SignupResponse {
    #[serde(rename = "objectId")]
    pub object_id: String,
    #[serde(rename = "sessionToken")]
    pub session_token: String,
    #[serde(
        rename = "createdAt",
        deserialize_with = "deserialize_string_to_parse_date"
    )]
    pub created_at: ParseDate,
}

// Request body for user signup
/// Represents the data required to sign up a new user.
///
/// This struct is typically serialized and sent as the body of a signup request.
/// It includes `username`, `password`, and an optional `email`.
/// Additional fields can be included by using a more generic type like `ParseObject` or `HashMap<String, Value>`
/// with the `signup` method if the server is configured to accept them.
#[derive(Serialize, Debug)]
pub struct SignupRequest<'a> {
    pub username: &'a str,
    pub password: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<&'a str>,
}

// Request body for user login
/// Represents the data required to log in an existing user.
///
/// This struct is typically serialized and sent as the body of a login request.
/// It includes `username` and `password`.
#[derive(Serialize, Debug)]
pub struct LoginRequest<'a> {
    pub username: &'a str,
    pub password: &'a str,
}

// Request body for password reset request
/// Represents the data required to request a password reset for a user.
///
/// This struct is typically serialized and sent as the body of a password reset request.
/// It includes the `email` address of the user requesting the reset.
#[derive(Serialize, Debug)]
pub struct PasswordResetRequest<'a> {
    pub email: &'a str,
}

/// Provides methods for managing user authentication and user-specific operations.
///
/// An instance of `ParseUserHandle` is obtained by calling the [`user()`](crate::Parse::user)
/// method on a `Parse` instance. It allows for operations such as signing up new users,
/// logging in existing users, fetching the current user's details, logging out, and requesting password resets.
///
/// The handle maintains a mutable reference to the `Parse` to update its session state (e.g., `session_token`)
/// upon successful login or signup, and to clear it on logout.
pub struct ParseUserHandle<'a> {
    client: &'a mut crate::Parse,
}

impl ParseUserHandle<'_> {
    /// Creates a new `ParseUserHandle`.
    ///
    /// This constructor is typically called by `Parse::user()`.
    ///
    /// # Arguments
    ///
    /// * `client`: A mutable reference to the `Parse` instance that this handle will operate upon.
    pub fn new(client: &mut crate::Parse) -> ParseUserHandle<'_> {
        ParseUserHandle { client }
    }

    // User management methods
    /// Signs up a new user with the Parse Server.
    ///
    /// This method sends the provided user data to the `/users` endpoint. Upon successful signup,
    /// the Parse Server returns the new user's `objectId`, a `sessionToken`, and `createdAt` timestamp.
    /// The `sessionToken` is automatically stored in the `Parse` instance, making the new user
    /// the current authenticated user for subsequent requests.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The type of the `user_data` argument. This type must implement `Serialize`, `Send`, and `Sync`.
    ///   Commonly, this will be [`SignupRequest`](crate::user::SignupRequest) for standard username/password/email signups,
    ///   but can also be a `ParseObject` or a `HashMap<String, Value>` if you need to include additional
    ///   custom fields during signup (assuming your Parse Server is configured to allow this).
    ///
    /// # Arguments
    ///
    /// * `user_data`: A reference to the data for the new user. This typically includes `username` and `password`,
    ///   and can optionally include `email` and other custom fields.
    ///
    /// # Returns
    ///
    /// A `Result` containing a [`SignupResponse`](crate::user::SignupResponse) if the signup is successful,
    /// or a `ParseError` if the signup fails (e.g., username taken, invalid data, network issue).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use parse_rs::{Parse, ParseError, user::SignupRequest};
    /// use serde_json::Value;
    /// use std::collections::HashMap;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), ParseError> {
    /// # let server_url = std::env::var("PARSE_SERVER_URL").unwrap_or_else(|_| "http://localhost:1338/parse".to_string());
    /// # let app_id = std::env::var("PARSE_APP_ID").unwrap_or_else(|_| "myAppId".to_string());
    /// # let mut client = Parse::new(&server_url, &app_id, None, None, None)?;
    ///
    /// // Example 1: Using SignupRequest for standard fields
    /// let signup_details = SignupRequest {
    ///     username: "new_user_1",
    ///     password: "securePassword123",
    ///     email: Some("user1@example.com"),
    /// };
    ///
    /// match client.user().signup(&signup_details).await {
    ///     Ok(response) => {
    ///         println!("User '{}' signed up successfully! ObjectId: {}, Session Token: {}",
    ///                  signup_details.username, response.object_id, response.session_token);
    ///         assert_eq!(client.session_token(), Some(response.session_token.as_str()));
    ///     }
    ///     Err(e) => eprintln!("Signup failed for user '{}': {}", signup_details.username, e),
    /// }
    ///
    /// // Example 2: Using HashMap for additional custom fields (if server allows)
    /// let mut custom_signup_data = HashMap::new();
    /// custom_signup_data.insert("username".to_string(), Value::String("new_user_2".to_string()));
    /// custom_signup_data.insert("password".to_string(), Value::String("anotherSecurePass456".to_string()));
    /// custom_signup_data.insert("email".to_string(), Value::String("user2@example.com".to_string()));
    /// custom_signup_data.insert("customField".to_string(), Value::String("customValue".to_string()));
    /// custom_signup_data.insert("age".to_string(), Value::Number(30.into()));
    ///
    /// // match client.user().signup(&custom_signup_data).await {
    /// //     Ok(response) => {
    /// //         println!("User with custom data signed up! ObjectId: {}, Session: {}",
    /// //                  response.object_id, response.session_token);
    /// //     }
    /// //     Err(e) => eprintln!("Custom signup failed: {}", e),
    /// // }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn signup<T: Serialize + Send + Sync>(
        &mut self,
        user_data: &T, // Changed from SignupRequest to generic T for flexibility if ParseObject is used
    ) -> Result<SignupResponse, ParseError> {
        // Endpoint for signup is typically "users"
        match self
            .client
            ._request::<_, SignupResponse>(Method::POST, "users", Some(user_data), false, None)
            .await
        {
            Ok(response) => {
                // Assuming SignupResponse contains a session_token field
                self.client.session_token = Some(response.session_token.clone());
                Ok(response)
            }
            Err(e) => Err(e),
        }
    }

    /// Logs in an existing user with the Parse Server.
    ///
    /// This method sends the provided user credentials (typically username and password) to the `/login` endpoint.
    /// Upon successful login, the Parse Server returns the full user object, including a `sessionToken`.
    /// This `sessionToken` is automatically stored in the `Parse` instance, making the user
    /// the current authenticated user for subsequent requests.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The type of the `user_data` argument. This type must implement `Serialize`, `Send`, and `Sync`.
    ///   Commonly, this will be [`LoginRequest`](crate::user::LoginRequest) for standard username/password logins.
    ///   It could also be a `HashMap<String, Value>` if the server supports other login mechanisms via the same endpoint,
    ///   though this is less common for the standard `/login` route.
    ///
    /// # Arguments
    ///
    /// * `user_data`: A reference to the credentials for the user to log in. This typically includes `username` and `password`.
    ///
    /// # Returns
    ///
    /// A `Result` containing the full [`ParseUser`](crate::user::ParseUser) object if the login is successful,
    /// or a `ParseError` if the login fails (e.g., invalid credentials, user not found, network issue).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use parse_rs::{Parse, ParseError, user::LoginRequest};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), ParseError> {
    /// # let server_url = std::env::var("PARSE_SERVER_URL").unwrap_or_else(|_| "http://localhost:1338/parse".to_string());
    /// # let app_id = std::env::var("PARSE_APP_ID").unwrap_or_else(|_| "myAppId".to_string());
    /// # let mut client = Parse::new(&server_url, &app_id, None, None, None)?;
    ///
    /// // Assume "test_user" was previously signed up
    /// let login_details = LoginRequest {
    ///     username: "test_user",
    ///     password: "password123",
    /// };
    ///
    /// match client.user().login(&login_details).await {
    ///     Ok(logged_in_user) => {
    ///         println!("User '{}' logged in successfully! Session Token: {}",
    ///                  logged_in_user.username,
    ///                  logged_in_user.session_token.as_deref().unwrap_or("N/A"));
    ///         assert_eq!(client.session_token(), logged_in_user.session_token.as_deref());
    ///         assert_eq!(logged_in_user.username, "test_user");
    ///     }
    ///     Err(e) => eprintln!("Login failed for user '{}': {}", login_details.username, e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn login<T: Serialize + Send + Sync>(
        &mut self,
        user_data: &T, // Changed from LoginRequest to generic T
    ) -> Result<ParseUser, ParseError> {
        // Endpoint for login is typically "login"
        match self
            .client
            ._request::<_, ParseUser>(Method::POST, "login", Some(user_data), false, None)
            .await
        {
            Ok(user_response) => {
                self.client.session_token = user_response.session_token.clone();
                Ok(user_response)
            }
            Err(e) => Err(e),
        }
    }

    // GET /users/me - requires session token
    /// Fetches the details of the currently authenticated user.
    ///
    /// This method makes a GET request to the `/users/me` endpoint, which requires a valid
    /// session token to be present in the `Parse` (set automatically after a successful
    /// login or signup).
    ///
    /// If no session token is available in the client, this method will return a
    /// `ParseError::SessionTokenMissing` error without making a network request.
    ///
    /// # Returns
    ///
    /// A `Result` containing the [`ParseUser`](crate::user::ParseUser) object for the currently
    /// authenticated user if successful, or a `ParseError` if the request fails (e.g., session token
    /// is invalid or expired, network issue, or no session token is present).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use parse_rs::{Parse, ParseError, user::LoginRequest};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), ParseError> {
    /// # let server_url = std::env::var("PARSE_SERVER_URL").unwrap_or_else(|_| "http://localhost:1338/parse".to_string());
    /// # let app_id = std::env::var("PARSE_APP_ID").unwrap_or_else(|_| "myAppId".to_string());
    /// # let mut client = Parse::new(&server_url, &app_id, None, None, None)?;
    ///
    /// // First, log in a user (or sign them up)
    /// // let login_details = LoginRequest { username: "test_user", password: "password123" };
    /// // client.user().login(&login_details).await?;
    ///
    /// if client.is_authenticated() {
    ///     match client.user().me().await {
    ///         Ok(current_user) => {
    ///             println!("Current user: {}, Email: {:?}",
    ///                      current_user.username, current_user.email.as_deref().unwrap_or("N/A"));
    ///             // The sessionToken field in the returned ParseUser object from /users/me
    ///             // might be the same or a new one depending on server configuration.
    ///             // The client's session token remains the one used for the request.
    ///         }
    ///         Err(e) => eprintln!("Failed to fetch current user: {}", e),
    ///     }
    /// } else {
    ///     println!("No user is currently authenticated.");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn me(&self) -> Result<ParseUser, ParseError> {
        // Removed &mut self as it only reads session token
        if self.client.session_token.is_none() {
            return Err(ParseError::SessionTokenMissing);
        }
        // current_user does not take a body
        self.client
            ._request(Method::GET, "users/me", None::<&Value>, false, None)
            .await
    }

    // POST /logout - requires session token
    /// Logs out the currently authenticated user.
    ///
    /// This method sends a POST request to the `/logout` endpoint using the current session token
    /// stored in the `Parse`. If successful, the Parse Server invalidates the session token.
    /// This method also clears the `session_token` from the `Parse` instance, effectively
    /// ending the current user's session on the client-side as well.
    ///
    /// If no session token is available in the client, this method will return a
    /// `ParseError::SessionTokenMissing` error without making a network request.
    ///
    /// # Returns
    ///
    /// A `Result` containing `()` (an empty tuple) if the logout is successful, or a `ParseError`
    /// if the request fails (e.g., session token already invalid, network issue, or no session token
    /// is present).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use parse_rs::{Parse, ParseError, user::LoginRequest};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), ParseError> {
    /// # let server_url = std::env::var("PARSE_SERVER_URL").unwrap_or_else(|_| "http://localhost:1338/parse".to_string());
    /// # let app_id = std::env::var("PARSE_APP_ID").unwrap_or_else(|_| "myAppId".to_string());
    /// # let mut client = Parse::new(&server_url, &app_id, None, None, None)?;
    ///
    /// // First, ensure a user is logged in
    /// // let login_details = LoginRequest { username: "test_user", password: "password123" };
    /// // client.user().login(&login_details).await?;
    ///
    /// if client.is_authenticated() {
    ///     println!("User is authenticated with token: {:?}", client.session_token());
    ///     match client.user().logout().await {
    ///         Ok(_) => {
    ///             println!("User logged out successfully.");
    ///             assert!(!client.is_authenticated(), "Client should not be authenticated after logout.");
    ///             assert!(client.session_token().is_none(), "Session token should be cleared after logout.");
    ///         }
    ///         Err(e) => eprintln!("Logout failed: {}", e),
    ///     }
    /// } else {
    ///     println!("No user was logged in to log out.");
    /// }
    ///
    /// // Attempting logout again when not authenticated should fail (or do nothing gracefully)
    /// // match client.user().logout().await {
    /// //     Err(ParseError::SessionTokenMissing) => println!("Correctly failed: Session token missing for logout."),
    /// //     _ => eprintln!("Logout behavior when not authenticated is unexpected."),
    /// // }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn logout(&mut self) -> Result<(), ParseError> {
        if self.client.session_token.is_none() {
            return Err(ParseError::SessionTokenMissing);
        }
        // Logout does not take a body and expects an empty JSON {} or specific success response
        let result: Result<Value, _> = self
            .client
            ._request(Method::POST, "logout", None::<&Value>, false, None)
            .await;
        match result {
            Ok(_value) => {
                // Parse server returns an empty JSON object {} on successful logout
                self.client.session_token = None;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    // POST /requestPasswordReset - public, no session token needed
    /// Requests a password reset email to be sent to the user associated with the given email address.
    ///
    /// This method sends a POST request to the `/requestPasswordReset` endpoint with the user's email.
    /// The Parse Server then handles sending the password reset email if a user with that email exists.
    /// This operation does not require a session token and can be called publicly.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The type of the `email_data` argument. This type must implement `Serialize`, `Send`, and `Sync`.
    ///   Commonly, this will be [`PasswordResetRequest`](crate::user::PasswordResetRequest).
    ///
    /// # Arguments
    ///
    /// * `email_data`: A reference to the data containing the email address for the password reset request.
    ///
    /// # Returns
    ///
    /// A `Result` containing `()` (an empty tuple) if the request is successfully sent to the server,
    /// or a `ParseError` if the request fails (e.g., invalid email format, network issue).
    /// Note: A successful response means the request was accepted by the server, not necessarily that
    /// a user with that email exists or that an email was actually sent (to prevent leaking user information).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use parse_rs::{Parse, ParseError, user::PasswordResetRequest};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), ParseError> {
    /// # let server_url = std::env::var("PARSE_SERVER_URL").unwrap_or_else(|_| "http://localhost:1338/parse".to_string());
    /// # let app_id = std::env::var("PARSE_APP_ID").unwrap_or_else(|_| "myAppId".to_string());
    /// # let mut client = Parse::new(&server_url, &app_id, None, None, None)?;
    ///
    /// let email_for_reset = "user_to_reset@example.com";
    /// let reset_request_data = PasswordResetRequest { email: email_for_reset };
    ///
    /// match client.user().request_password_reset(&reset_request_data).await {
    ///     Ok(_) => println!("Password reset request sent for email: {}", email_for_reset),
    ///     Err(e) => eprintln!("Failed to send password reset request for email '{}': {}", email_for_reset, e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn request_password_reset<T: Serialize + Send + Sync>(
        &self,
        email_data: &T, // Changed from PasswordResetRequest to generic T
    ) -> Result<(), ParseError> {
        // request_password_reset expects an empty JSON {} or specific success response
        let result: Result<Value, _> = self
            .client
            ._request(
                Method::POST,
                "requestPasswordReset",
                Some(email_data),
                false,
                None,
            )
            .await;
        match result {
            Ok(_value) => Ok(()), // Expects empty {} on success
            Err(e) => Err(e),
        }
    }

    // GET /users/me - but with a different session token to become that user
    // This is a tricky one. The `become` operation itself is a GET to /users/me, but authenticated with the *target* session token.
    // The client's current session token is replaced upon success.
    /// Allows the current client to "become" another user by using that user's session token.
    ///
    /// This method makes a GET request to `/users/me`, but authenticates it using the
    /// `session_token_to_become` provided as an argument, instead of the client's current session token.
    /// If the provided session token is valid, the server responds with the details of the user
    /// associated with that token. Crucially, upon a successful response, this method **replaces**
    /// the `Parse`'s current `session_token` with `session_token_to_become`.
    ///
    /// This is a powerful operation and should be used with caution, typically in administrative
    /// contexts or when implementing features like "Log in as user" for support purposes.
    /// The client performing this operation must have access to the target user's session token.
    ///
    /// # Arguments
    ///
    /// * `session_token_to_become`: A string slice representing the session token of the user
    ///   to become.
    ///
    /// # Returns
    ///
    /// A `Result` containing the [`ParseUser`](crate::user::ParseUser) object for the user
    /// whose session token was provided, if the operation is successful. Returns a `ParseError`
    /// if the provided session token is invalid, expired, or if any other error occurs during
    /// the request.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use parse_rs::{Parse, ParseError};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), ParseError> {
    /// # let server_url = std::env::var("PARSE_SERVER_URL").unwrap_or_else(|_| "http://localhost:1338/parse".to_string());
    /// # let app_id = std::env::var("PARSE_APP_ID").unwrap_or_else(|_| "myAppId".to_string());
    /// # let mut client = Parse::new(&server_url, &app_id, None, None, None)?;
    ///
    /// // Assume `target_user_session_token` is a valid session token for another user, obtained securely.
    /// let target_user_session_token = "r:someValidSessionTokenForAnotherUser";
    ///
    /// println!("Client's current session token before 'become': {:?}", client.session_token());
    ///
    /// match client.user().become_user(target_user_session_token).await {
    ///     Ok(newly_become_user) => {
    ///         println!("Successfully became user '{}' (ID: {}).",
    ///                  newly_become_user.username,
    ///                  newly_become_user.object_id.as_deref().unwrap_or("N/A"));
    ///         println!("Client's session token is now: {:?}", client.session_token());
    ///         assert_eq!(client.session_token(), Some(target_user_session_token));
    ///     }
    ///     Err(e) => {
    ///         eprintln!("Failed to become user with token '{}': {}", target_user_session_token, e);
    ///         // Client's original session token should be restored if 'become' failed.
    ///         println!("Client's session token after failed 'become': {:?}", client.session_token());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn become_user(
        &mut self,
        session_token_to_become: &str,
    ) -> Result<ParseUser, ParseError> {
        // Temporarily override client's auth for this specific request
        // This requires a way to make _request use a specific session token, or a dedicated _request_with_token method.
        // Current _request uses self.session_token or master_key.
        // A simple way is to clone the client, set the token, make the request, then update original client.
        // Or, modify _request to accept an optional override token.

        // For now, let's assume _request needs modification or we use a more direct approach for this one-off auth.
        // The most straightforward modification to _request would be to allow an optional override token.
        // Let's simulate this by creating a temporary request builder here, which is not ideal as it bypasses _request's error handling.

        // Ideal approach: Modify _request to handle an override token.
        // Fallback: Direct reqwest call for this specific case.
        // Given current _request, this is hard to do cleanly without modifying _request signature or logic.

        // Let's assume for now that we will add a specialized method or enhance _request later.
        // For this refactor, we'll placeholder it or acknowledge it needs a more specific implementation.
        // One way to achieve this with current _request: temporarily set self.session_token, then revert.
        let original_token = self.client.session_token.clone();
        self.client.session_token = Some(session_token_to_become.to_string());

        match self
            .client
            ._request(Method::GET, "users/me", None::<&Value>, false, None)
            .await
        {
            Ok(user_data) => {
                // self.session_token is already set to session_token_to_become, so this is correct.
                Ok(user_data)
            }
            Err(e) => {
                self.client.session_token = original_token; // Revert on error
                Err(e)
            } // If successful, self.session_token remains as session_token_to_become
        }
    }
}
