use crate::acl::ParseACL;
use crate::error::ParseError;
use crate::object::{deserialize_string_to_option_parse_date, deserialize_string_to_parse_date};
use crate::types::common::{Pointer, RelationOp};
use crate::types::ParseDate; // Assuming ParseDate is in crate::types
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Represents a Parse Role object, used for grouping users and other roles to manage permissions.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ParseRole {
    /// The unique identifier for the role object.
    #[serde(rename = "objectId", skip_serializing_if = "Option::is_none")]
    pub object_id: Option<String>,
    /// The timestamp when the role was created.
    #[serde(
        rename = "createdAt",
        deserialize_with = "deserialize_string_to_option_parse_date",
        skip_serializing_if = "Option::is_none"
    )]
    pub created_at: Option<ParseDate>,
    /// The timestamp when the role was last updated.
    #[serde(
        rename = "updatedAt",
        deserialize_with = "deserialize_string_to_option_parse_date",
        skip_serializing_if = "Option::is_none"
    )]
    pub updated_at: Option<ParseDate>,
    /// The name of the role. This is required and must be unique.
    /// It can only be set upon creation.
    pub name: String,
    /// The Access Control List for this role, determining who can read or write it.
    #[serde(rename = "ACL")]
    pub acl: ParseACL,
    // The 'users' and 'roles' fields are relations and are managed via specific API calls
    // or through ParseQuery using the $relatedTo operator. They are not typically part of
    // the direct object representation unless explicitly included and expanded by the server,
    // which is not the default behavior for direct role object retrieval.
    /// Placeholder for any other custom fields that might be on a Role object.
    /// While the _Role class is special, Parse Server might allow adding custom fields.
    #[serde(flatten, default, skip_serializing_if = "HashMap::is_empty")]
    pub other_fields: HashMap<String, Value>,
}

/// Represents the data required to create a new Parse Role.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct NewParseRole {
    /// The name for the new role. Must be unique and is required.
    pub name: String,
    /// The Access Control List for the new role. Required.
    #[serde(rename = "ACL")]
    pub acl: ParseACL,
    // TODO: Consider how to represent initial users and roles for creation.
    // This might involve a different structure or passing relation operations.
    // For now, keeping it simple: create role, then add relations.
}

/// Represents the server's response when a new role is successfully created.
#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub(crate) struct CreateRoleResponse {
    #[serde(rename = "objectId")]
    pub object_id: String,
    #[serde(
        rename = "createdAt",
        deserialize_with = "deserialize_string_to_parse_date"
    )]
    pub created_at: ParseDate, // Or String if ParseDate deserialization needs adjustment
}

impl crate::Parse {
    // Role management methods

    /// Creates a new Role on the Parse Server.
    ///
    /// # Arguments
    /// * `new_role`: A `NewParseRole` struct containing the name and ACL for the new role.
    ///
    /// # Returns
    /// A `Result` containing the created `ParseRole` or a `ParseError`.
    /// Note: The `users` and `roles` relations are not populated in the returned object.
    /// They need to be managed via separate relation operations or queries.
    pub async fn create_role(&self, new_role: &NewParseRole) -> Result<ParseRole, ParseError> {
        let endpoint = "roles";
        // Roles are typically managed with Master Key for security.
        // For now, defaulting to false, but this might need to be true or configurable.
        let use_master_key = self.master_key.is_some();

        let response: CreateRoleResponse = self
            ._request(
                Method::POST,
                endpoint,
                Some(new_role),
                use_master_key, // Use master key if available for role creation
                None,           // No specific session token for role creation itself usually
            )
            .await?;

        Ok(ParseRole {
            object_id: Some(response.object_id),
            created_at: Some(response.created_at),
            updated_at: None, // Not provided by create response
            name: new_role.name.clone(),
            acl: new_role.acl.clone(),
            other_fields: Default::default(),
        })
    }

    /// Retrieves a specific Role by its objectId.
    ///
    /// # Arguments
    /// * `object_id`: The objectId of the role to retrieve.
    ///
    /// # Returns
    /// A `Result` containing the `ParseRole` or a `ParseError`.
    /// Note: The `users` and `roles` relations are not populated by this call.
    pub async fn get_role(&self, object_id: &str) -> Result<ParseRole, ParseError> {
        let endpoint = format!("roles/{}", object_id);
        // Reading a role might be allowed with different auth types depending on ACL.
        // Defaulting to standard auth (session token if present, or JS/REST key).
        // Master key can also be used if needed for roles with restrictive read ACLs.
        let use_master_key = false; // Or determine based on needs/ACLs if this becomes more complex

        self._request(
            Method::GET,
            &endpoint,
            None::<&()>,
            use_master_key,
            self.session_token.as_deref(),
        )
        .await
    }

    /// Deletes a specific Role by its objectId.
    ///
    /// # Arguments
    /// * `object_id`: The objectId of the role to delete.
    ///
    /// # Returns
    /// A `Result` indicating success (`Ok(())`) or a `ParseError`.
    /// This operation typically requires the Master Key or appropriate user permissions.
    pub async fn delete_role(&self, object_id: &str) -> Result<(), ParseError> {
        let endpoint = format!("roles/{}", object_id);
        // Deleting roles typically requires Master Key or specific user permissions.
        // Prioritize Master Key if available.
        let use_master_key = self.master_key.is_some();
        let session_token_to_use = if use_master_key {
            None
        } else {
            self.session_token.as_deref()
        };

        let _response: serde_json::Value = self
            ._request(
                Method::DELETE,
                &endpoint,
                None::<&()>,
                use_master_key,
                session_token_to_use,
            )
            .await?;
        // Successful DELETE usually returns an empty body or a simple confirmation
        // that serde_json::Value can handle. We don't need to use the value.
        Ok(())
    }

    /// Adds users to a specific Role.
    ///
    /// # Arguments
    /// * `role_id`: The objectId of the Role to modify.
    /// * `user_ids`: A slice of objectIds of the Users to add to the role.
    ///
    /// # Returns
    /// A `Result` containing the `ParseDate` of the update or a `ParseError`.
    /// This operation typically requires the Master Key.
    pub async fn add_users_to_role(
        &self,
        role_id: &str,
        user_ids: &[&str],
    ) -> Result<ParseDate, ParseError> {
        if user_ids.is_empty() {
            // Or return current updatedAt if we fetch the role first? For now, early return.
            return Err(ParseError::InvalidInput(
                "user_ids cannot be empty for AddRelation.".to_string(),
            ));
        }
        let endpoint = format!("roles/{}", role_id);
        let pointers: Vec<Pointer> = user_ids
            .iter()
            .map(|id| Pointer::new("_User", id.to_string()))
            .collect();

        let relation_op = RelationOp::add(&pointers);
        let body = serde_json::json!({ "users": relation_op });

        // Modifying role relations typically requires Master Key.
        let use_master_key = self.master_key.is_some();
        if !use_master_key {
            // Potentially return an error or log a warning, as this operation might fail without master key
            // For now, proceed, but server will likely reject if ACLs are restrictive and no master key.
        }
        let session_token_to_use = if use_master_key {
            None
        } else {
            self.session_token.as_deref()
        };

        #[derive(Deserialize)]
        struct UpdateResponse {
            #[serde(rename = "updatedAt")]
            updated_at: String,
        }

        let response: UpdateResponse = self
            ._request(
                Method::PUT,
                &endpoint,
                Some(&body),
                use_master_key,
                session_token_to_use,
            )
            .await?;

        Ok(ParseDate::new(response.updated_at))
    }

    /// Removes users from a specific Role.
    ///
    /// # Arguments
    /// * `role_id`: The objectId of the Role to modify.
    /// * `user_ids`: A slice of objectIds of the Users to remove from the role.
    ///
    /// # Returns
    /// A `Result` containing the `ParseDate` of the update or a `ParseError`.
    /// This operation typically requires the Master Key.
    pub async fn remove_users_from_role(
        &self,
        role_id: &str,
        user_ids: &[&str],
    ) -> Result<ParseDate, ParseError> {
        if user_ids.is_empty() {
            return Err(ParseError::InvalidInput(
                "user_ids cannot be empty for RemoveRelation.".to_string(),
            ));
        }
        let endpoint = format!("roles/{}", role_id);
        let pointers: Vec<Pointer> = user_ids
            .iter()
            .map(|id| Pointer::new("_User", id.to_string()))
            .collect();

        let relation_op = RelationOp::remove(&pointers);
        let body = serde_json::json!({ "users": relation_op });

        let use_master_key = self.master_key.is_some();
        let session_token_to_use = if use_master_key {
            None
        } else {
            self.session_token.as_deref()
        };

        #[derive(Deserialize)]
        struct UpdateResponse {
            #[serde(rename = "updatedAt")]
            updated_at: String,
        }

        let response: UpdateResponse = self
            ._request(
                Method::PUT,
                &endpoint,
                Some(&body),
                use_master_key,
                session_token_to_use,
            )
            .await?;

        Ok(ParseDate::new(response.updated_at))
    }

    /// Adds child roles to a specific (parent) Role.
    ///
    /// # Arguments
    /// * `role_id`: The objectId of the parent Role to modify.
    /// * `child_role_ids`: A slice of objectIds of the child Roles to add to the parent role.
    ///
    /// # Returns
    /// A `Result` containing the `ParseDate` of the update or a `ParseError`.
    /// This operation typically requires the Master Key.
    pub async fn add_child_roles_to_role(
        &self,
        role_id: &str,
        child_role_ids: &[&str],
    ) -> Result<ParseDate, ParseError> {
        if child_role_ids.is_empty() {
            return Err(ParseError::InvalidInput(
                "child_role_ids cannot be empty for AddRelation.".to_string(),
            ));
        }
        let endpoint = format!("roles/{}", role_id);
        let pointers: Vec<Pointer> = child_role_ids
            .iter()
            .map(|id| Pointer::new("_Role", id.to_string()))
            .collect();

        let relation_op = RelationOp::add(&pointers);
        let body = serde_json::json!({ "roles": relation_op });

        let use_master_key = self.master_key.is_some();
        let session_token_to_use = if use_master_key {
            None
        } else {
            self.session_token.as_deref()
        };

        #[derive(Deserialize)]
        struct UpdateResponse {
            #[serde(rename = "updatedAt")]
            updated_at: String,
        }

        let response: UpdateResponse = self
            ._request(
                Method::PUT,
                &endpoint,
                Some(&body),
                use_master_key,
                session_token_to_use,
            )
            .await?;

        Ok(ParseDate::new(response.updated_at))
    }

    /// Removes child roles from a specific (parent) Role.
    ///
    /// # Arguments
    /// * `role_id`: The objectId of the parent Role to modify.
    /// * `child_role_ids`: A slice of objectIds of the child Roles to remove from the parent role.
    ///
    /// # Returns
    /// A `Result` containing the `ParseDate` of the update or a `ParseError`.
    /// This operation typically requires the Master Key.
    pub async fn remove_child_roles_from_role(
        &self,
        role_id: &str,
        child_role_ids: &[&str],
    ) -> Result<ParseDate, ParseError> {
        let endpoint = format!("roles/{}", role_id);
        let pointers: Vec<Pointer> = child_role_ids
            .iter()
            .map(|&id| Pointer::new("_Role", id)) // Corrected to _Role
            .collect();
        let relation_op = RelationOp::remove(&pointers); // Use remove
        let mut body = std::collections::HashMap::new();
        body.insert("roles", relation_op); // The field name for role-to-role relations is "roles"

        // Define a local struct for the expected response, similar to add_child_roles_to_role
        #[derive(Deserialize)]
        struct UpdateResponse {
            #[serde(rename = "updatedAt")]
            updated_at: String,
        }

        let response: UpdateResponse = self
            ._request(
                Method::PUT,
                &endpoint,
                Some(&body),
                true, // use_master_key
                None, // session_token
            )
            .await?;
        Ok(ParseDate::new(response.updated_at))
    }
}
