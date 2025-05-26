// src/user.rs

use crate::object::{deserialize_string_to_option_parse_date, deserialize_string_to_parse_date};
use crate::types::ParseDate;
use crate::ParseError;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
#[derive(Serialize, Debug)]
pub struct SignupRequest<'a> {
    pub username: &'a str,
    pub password: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<&'a str>,
}

// Request body for user login
#[derive(Serialize, Debug)]
pub struct LoginRequest<'a> {
    pub username: &'a str,
    pub password: &'a str,
}

// Request body for password reset request
#[derive(Serialize, Debug)]
pub struct PasswordResetRequest<'a> {
    pub email: &'a str,
}

pub struct ParseUserHandle<'a> {
    client: &'a mut crate::Parse,
}

impl ParseUserHandle<'_> {
    pub fn new(client: &mut crate::Parse) -> ParseUserHandle<'_> {
        ParseUserHandle { client }
    }

    // User management methods
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
