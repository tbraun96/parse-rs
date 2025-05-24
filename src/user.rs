// src/user.rs

use crate::types::ParseDate;
use serde::{Deserialize, Serialize};

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
    #[serde(rename = "createdAt")]
    pub created_at: Option<ParseDate>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<ParseDate>,
}

// New struct for signup response
#[derive(Debug, Deserialize, Clone)]
pub struct SignupResponse {
    #[serde(rename = "objectId")]
    pub object_id: String,
    #[serde(rename = "sessionToken")]
    pub session_token: String,
    #[serde(rename = "createdAt")]
    pub created_at: ParseDate,
}

// Request body for user signup
#[derive(Serialize, Debug)]
pub(crate) struct SignupRequest<'a> {
    pub username: &'a str,
    pub password: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<&'a str>,
}

// Request body for user login
#[derive(Serialize, Debug)]
pub(crate) struct LoginRequest<'a> {
    pub username: &'a str,
    pub password: &'a str,
}

// Request body for password reset request
#[derive(Serialize, Debug)]
pub(crate) struct PasswordResetRequest<'a> {
    pub email: &'a str,
}
