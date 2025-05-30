// src/installation.rs
use crate::client::Parse;
use crate::error::ParseError;
use crate::object::{CreateObjectResponse, UpdateObjectResponse};
use crate::types::common::EmptyResponse;
use crate::ParseACL;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Represents the type of device for an installation.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    #[default]
    Js, // Web or JavaScript (Default)
    Ios,
    Android,
    Winphone, // Windows Phone
    Macos,
    Windows,
    Linux,
    Embedded,      // For other embedded systems
    Other(String), // For custom device types
}

/// Represents a new Parse Installation to be created.
#[derive(Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct NewParseInstallation {
    pub device_type: DeviceType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installation_id: Option<String>, // Client-generated unique ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_identifier: Option<String>, // e.g., bundle ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_zone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channels: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale_identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push_type: Option<String>, // e.g., "gcm" for Android
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gcm_sender_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acl: Option<ParseACL>,
    // You can add other custom fields as needed using a HashMap or by extending this struct.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<HashMap<String, Value>>,
}

impl NewParseInstallation {
    pub fn new(device_type: DeviceType) -> Self {
        Self {
            device_type,
            ..Default::default()
        }
    }
}

/// Represents a Parse Installation object retrieved from the server.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RetrievedParseInstallation {
    #[serde(rename = "objectId")]
    pub object_id: String,
    #[serde(rename = "createdAt")]
    pub created_at: String, // Parse Server returns these as ISO strings for Installation class
    #[serde(rename = "updatedAt")]
    pub updated_at: String, // Parse Server returns these as ISO strings for Installation class
    pub device_type: DeviceType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_zone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channels: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale_identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gcm_sender_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acl: Option<ParseACL>,
    // Captures any other fields returned by the server not explicitly defined.
    #[serde(flatten)]
    pub custom_fields: HashMap<String, Value>,
}

/// Represents the fields that can be updated on an existing Parse Installation.
/// All fields are optional, allowing for partial updates.
#[derive(Serialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateParseInstallation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_type: Option<DeviceType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_identifier: Option<String>, // e.g., com.example.app
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_zone: Option<String>, // e.g., America/New_York
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale_identifier: Option<String>, // e.g., en-US
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channels: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push_type: Option<String>, // Only for specific push services like gcm
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gcm_sender_id: Option<String>, // For Android GCM
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acl: Option<ParseACL>,
    // For any other custom fields to update.
    // Use `serde_json::json!({ "customField": "value" })` or build a HashMap.
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
    pub custom_fields: HashMap<String, Value>,
}

impl UpdateParseInstallation {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Parse {
    /// Creates a new Installation object on the Parse Server.
    ///
    /// # Arguments
    /// * `installation_data`: A `NewParseInstallation` struct containing the data for the new installation.
    ///
    /// # Returns
    /// A `Result` containing a `CreateObjectResponse` (which includes `objectId` and `createdAt`) or a `ParseError`.
    pub async fn create_installation(
        &self,
        installation_data: &NewParseInstallation,
    ) -> Result<CreateObjectResponse, ParseError> {
        // Installations are typically created without a session token, but can be associated with a user later.
        // The _Installation class usually requires the Master Key, JS Key, or REST API Key for creation.
        let use_master_key = self.master_key.is_some();
        let session_token_to_use = None;

        self._request(
            Method::POST,
            "installations",
            Some(installation_data),
            use_master_key,
            session_token_to_use,
        )
        .await
    }

    /// Retrieves a specific Installation object by its objectId.
    ///
    /// # Arguments
    /// * `object_id`: The objectId of the installation to retrieve.
    ///
    /// # Returns
    /// A `Result` containing the `RetrievedParseInstallation` or a `ParseError`.
    pub async fn get_installation(
        &self,
        object_id: &str,
    ) -> Result<RetrievedParseInstallation, ParseError> {
        if object_id.is_empty() {
            return Err(ParseError::InvalidInput(
                "Object ID cannot be empty.".to_string(),
            ));
        }
        let endpoint = format!("installations/{}", object_id);
        // Retrieving an installation usually requires Master Key, JS Key, or REST API Key.
        // It's generally not tied to a user session for direct GET by ID.
        let use_master_key = self.master_key.is_some();
        let session_token_to_use = None;

        self._request(
            Method::GET,
            &endpoint,
            None::<Value>.as_ref(), // No body for GET
            use_master_key,
            session_token_to_use,
        )
        .await
    }

    /// Updates an existing Installation object on the Parse Server.
    ///
    /// # Arguments
    /// * `object_id`: The objectId of the installation to update.
    /// * `update_data`: An `UpdateParseInstallation` struct containing the fields to update.
    ///
    /// # Returns
    /// A `Result` containing an `UpdateObjectResponse` (which includes `updatedAt`) or a `ParseError`.
    pub async fn update_installation(
        &self,
        object_id: &str,
        update_data: &UpdateParseInstallation,
    ) -> Result<UpdateObjectResponse, ParseError> {
        if object_id.is_empty() {
            return Err(ParseError::InvalidInput(
                "Object ID cannot be empty.".to_string(),
            ));
        }
        let endpoint = format!("installations/{}", object_id);
        // Updating an installation usually requires Master Key, JS Key, or REST API Key.
        let use_master_key = self.master_key.is_some();
        let session_token_to_use = None;

        self._request(
            Method::PUT,
            &endpoint,
            Some(update_data),
            use_master_key,
            session_token_to_use,
        )
        .await
    }

    /// Deletes an Installation object from the Parse Server.
    ///
    /// # Arguments
    /// * `object_id`: The objectId of the installation to delete.
    ///
    /// # Returns
    /// A `Result` containing an `EmptyResponse` or a `ParseError`.
    pub async fn delete_installation(&self, object_id: &str) -> Result<EmptyResponse, ParseError> {
        if object_id.is_empty() {
            return Err(ParseError::InvalidInput(
                "Object ID cannot be empty.".to_string(),
            ));
        }
        let endpoint = format!("installations/{}", object_id);
        // Deleting an installation usually requires Master Key.
        let use_master_key = self.master_key.is_some();
        if !use_master_key {
            // Log a warning or return an error if master key is preferred/required by server rules
            log::warn!("Attempting to delete an installation without the master key. This might be restricted by server ACLs/CLPs.");
        }
        let session_token_to_use = None;

        self._request(
            Method::DELETE,
            &endpoint,
            None::<Value>.as_ref(), // No body for DELETE
            use_master_key,
            session_token_to_use,
        )
        .await
    }
}

// We'll add other impl Parse methods (update, delete) here in subsequent steps.
