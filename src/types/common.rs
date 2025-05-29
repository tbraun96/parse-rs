use serde::{Deserialize, Serialize};

/// Represents a Pointer to another Parse object.
/// Pointers are used to create relationships between objects.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Pointer {
    #[serde(rename = "__type")]
    pub __type: String, // Should always be "Pointer"
    #[serde(rename = "className")]
    pub class_name: String,
    #[serde(rename = "objectId")]
    pub object_id: String,
}

impl Pointer {
    /// Creates a new Pointer.
    pub fn new(class_name: impl Into<String>, object_id: impl Into<String>) -> Self {
        Pointer {
            __type: "Pointer".to_string(),
            class_name: class_name.into(),
            object_id: object_id.into(),
        }
    }
}

/// Represents a Parse Date type, which includes timezone information.
/// Parse stores dates in UTC.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ParseDate {
    #[serde(rename = "__type")]
    pub __type: String, // Should always be "Date"
    pub iso: String, // ISO 8601 format, e.g., "YYYY-MM-DDTHH:MM:SS.MMMZ"
}

impl ParseDate {
    /// Creates a new ParseDate from an ISO 8601 string.
    /// Note: This does not validate the string format.
    pub fn new(iso_string: impl Into<String>) -> Self {
        ParseDate {
            __type: "Date".to_string(),
            iso: iso_string.into(),
        }
    }

    // TODO: Add a method to create from chrono::DateTime<Utc>
    // TODO: Add a method to convert to chrono::DateTime<Utc>
}

/// Represents a relational operation (AddRelation, RemoveRelation).
#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct RelationOp<'a, T>
where
    T: Serialize,
{
    #[serde(rename = "__op")]
    op_type: &'static str,
    objects: &'a [T],
}

impl<'a, T> RelationOp<'a, T>
where
    T: Serialize,
{
    pub fn add(objects: &'a [T]) -> Self {
        RelationOp {
            op_type: "AddRelation",
            objects,
        }
    }

    pub fn remove(objects: &'a [T]) -> Self {
        RelationOp {
            op_type: "RemoveRelation",
            objects,
        }
    }
}

/// Represents a Parse Relation field on an object.
/// This indicates a one-to-many or many-to-many relationship.
/// The actual related objects are typically fetched via a separate query.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ParseRelation {
    #[serde(rename = "__type")]
    pub __type: String, // Should always be "Relation"
    #[serde(rename = "className")]
    pub class_name: String, // The target class of the relation
}

impl ParseRelation {
    /// Creates a new ParseRelation indicator.
    /// This is mostly for completeness in representing the type, as relations
    /// are primarily managed through operations and queries.
    pub fn new(class_name: impl Into<String>) -> Self {
        ParseRelation {
            __type: "Relation".to_string(),
            class_name: class_name.into(),
        }
    }
}

/// Represents different Parse Server API endpoints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Endpoint {
    Classes(String),                 // Class name
    Objects(String, Option<String>), // Class name, optional objectId
    Users,
    UsersLogin,
    UsersLogout,
    UsersMe,
    RequestPasswordReset,
    Roles,
    RolesSpecific(String), // Role objectId
    Schemas,
    SchemasSpecific(String), // Schema class name
    Files(String),           // File name
    Functions(String),       // Function name
    Config,
    Aggregate(String), // Class name for aggregate
                       // Add other endpoints as needed
}

impl Endpoint {
    /// Builds the full URL path for the endpoint.
    pub fn build_url(&self, base_path: &str) -> String {
        let path = match self {
            Endpoint::Classes(class_name) => format!("{}/classes/{}", base_path, class_name),
            Endpoint::Objects(class_name, Some(object_id)) => {
                format!("{}/classes/{}/{}", base_path, class_name, object_id)
            }
            Endpoint::Objects(class_name, None) => format!("{}/classes/{}", base_path, class_name),
            Endpoint::Users => format!("{}/users", base_path),
            Endpoint::UsersLogin => format!("{}/login", base_path),
            Endpoint::UsersLogout => format!("{}/logout", base_path),
            Endpoint::UsersMe => format!("{}/users/me", base_path),
            Endpoint::RequestPasswordReset => format!("{}/requestPasswordReset", base_path),
            Endpoint::Roles => format!("{}/roles", base_path),
            Endpoint::RolesSpecific(role_id) => format!("{}/roles/{}", base_path, role_id),
            Endpoint::Schemas => format!("{}/schemas", base_path),
            Endpoint::SchemasSpecific(class_name) => {
                format!("{}/schemas/{}", base_path, class_name)
            }
            Endpoint::Files(file_name) => format!("{}/files/{}", base_path, file_name),
            Endpoint::Functions(function_name) => {
                format!("{}/functions/{}", base_path, function_name)
            }
            Endpoint::Config => format!("{}/config", base_path),
            Endpoint::Aggregate(class_name) => format!("{}/aggregate/{}", base_path, class_name),
        };
        // Ensure no double slashes if base_path is just "/"
        path.replace("//", "/")
    }
}

/// Type alias for query parameters, typically a HashMap.
pub type QueryParams = std::collections::HashMap<String, String>;

/// Represents a generic list of results, commonly returned by find operations.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Results<T> {
    pub results: Vec<T>,
    // Add other common fields like 'count' if needed for pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,
}

/// Represents common data in response to update operations.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateResponseData {
    #[serde(rename = "updatedAt")]
    pub updated_at: String, // ISO 8601 format
    // Some update operations might return objectId, e.g., if it's a create-or-update
    #[serde(rename = "objectId", skip_serializing_if = "Option::is_none")]
    pub object_id: Option<String>,
}
