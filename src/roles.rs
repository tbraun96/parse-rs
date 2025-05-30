use serde::{Deserialize, Serialize};
use crate::acl::ParseACL;
use crate::types::date::ParseDate;

/// Represents a new Parse Role to be created.
#[derive(Debug, Serialize, Clone)]
pub struct NewParseRole {
    pub name: String,
    pub acl: ParseACL,
}

/// Represents a Parse Role object fetched from the server.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ParseRole {
    pub object_id: String,
    pub name: String,
    pub acl: ParseACL,
    pub created_at: ParseDate,
    pub updated_at: ParseDate,
    // Roles and Users are relations, handle later if needed for update/query
    // "users": {"__type": "Relation", "className": "_User"},
    // "roles": {"__type": "Relation", "className": "_Role"}
}

// This struct is for the actual response from POST /roles
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateRoleServerResponse {
    pub object_id: String,
    pub created_at: ParseDate,
}
