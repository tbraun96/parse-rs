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
