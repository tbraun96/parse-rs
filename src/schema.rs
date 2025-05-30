use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Represents the possible data types for a field in a Parse class schema.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum FieldType {
    String,
    Number,
    Boolean,
    Date,
    Object, // Generic JSON object
    Array,
    Pointer,
    Relation,
    File,
    GeoPoint,
    ACL,
    Bytes,
    Polygon,
    // Note: Add other types if Parse Server supports more that are relevant for schema definition.
}

/// Represents the schema definition for a single field within a Parse class.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct FieldSchema {
    /// The data type of the field.
    #[serde(rename = "type")]
    pub field_type: FieldType,

    /// For `Pointer` and `Relation` types, this specifies the target class name.
    #[serde(rename = "targetClass", skip_serializing_if = "Option::is_none")]
    pub target_class: Option<String>,

    /// Indicates if the field is required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,

    /// The default value for the field.
    #[serde(rename = "defaultValue", skip_serializing_if = "Option::is_none")]
    pub default_value: Option<Value>,
}

/// Represents the Class Level Permissions (CLP) for a Parse class schema.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ClassLevelPermissionsSchema {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get: Option<HashMap<String, bool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub find: Option<HashMap<String, bool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<HashMap<String, bool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create: Option<HashMap<String, bool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update: Option<HashMap<String, bool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<HashMap<String, bool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add_field: Option<HashMap<String, bool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_user_fields: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write_user_fields: Option<Vec<String>>,
}

/// Represents the schema for a Parse class, including its fields, CLP, and indexes.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum IndexFieldType {
    SortOrder(i32),
    Text(String),
    Other(Value),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ParseSchema {
    pub class_name: String,
    pub fields: HashMap<String, FieldSchema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class_level_permissions: Option<ClassLevelPermissionsSchema>,
    /// Indexes are represented as a map where the key is the index name
    /// and the value is another map from field name to sort order (1 for asc, -1 for desc).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indexes: Option<HashMap<String, HashMap<String, IndexFieldType>>>,
}

/// Represents the response structure when fetching all schemas.
#[derive(Debug, Deserialize, Clone)]
pub struct GetAllSchemasResponse {
    pub results: Vec<ParseSchema>,
}
