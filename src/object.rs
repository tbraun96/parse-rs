// src/object.rs

use crate::acl::ParseACL;
use crate::client::ParseClient;
use crate::types::date::ParseDate;
use crate::ParseError;
use serde::de::{DeserializeOwned, Deserializer};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

// Helper function to deserialize a string into Option<ParseDate>
pub fn deserialize_string_to_option_parse_date<'de, D>(
    deserializer: D,
) -> Result<Option<ParseDate>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    Ok(s.map(ParseDate::new))
}

// Helper function to deserialize a string into ParseDate
pub fn deserialize_string_to_parse_date<'de, D>(deserializer: D) -> Result<ParseDate, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    Ok(ParseDate::new(s))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseObject {
    #[serde(skip_serializing_if = "Option::is_none", rename = "objectId")]
    pub object_id: Option<String>,
    #[serde(
        deserialize_with = "deserialize_string_to_option_parse_date",
        skip_serializing_if = "Option::is_none",
        rename = "createdAt"
    )]
    pub created_at: Option<ParseDate>,
    #[serde(
        deserialize_with = "deserialize_string_to_option_parse_date",
        skip_serializing_if = "Option::is_none",
        rename = "updatedAt"
    )]
    pub updated_at: Option<ParseDate>,
    #[serde(flatten)]
    pub fields: HashMap<String, Value>,
    #[serde(rename = "ACL", skip_serializing_if = "Option::is_none")]
    pub acl: Option<ParseACL>,
    #[serde(skip_serializing, default)]
    // Should not be serialized, only used for context. Default if missing.
    pub class_name: String,
}

impl ParseObject {
    pub fn new(class_name: &str) -> Self {
        ParseObject {
            class_name: class_name.to_string(),
            fields: HashMap::new(),
            acl: None,
            object_id: None,
            created_at: None,
            updated_at: None,
        }
    }

    pub fn set<T: Serialize>(&mut self, field_name: &str, value: T) {
        self.fields
            .insert(field_name.to_string(), serde_json::to_value(value).unwrap());
    }

    pub fn get<T: DeserializeOwned>(&self, field_name: &str) -> Option<T> {
        self.fields
            .get(field_name)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    pub fn set_acl(&mut self, acl: ParseACL) {
        self.acl = Some(acl);
    }

    pub fn increment(&mut self, field_name: &str, amount: i64) {
        let op = json!({
            "__op": "Increment",
            "amount": amount
        });
        self.fields.insert(field_name.to_string(), op);
    }

    pub fn decrement(&mut self, field_name: &str, amount: i64) {
        self.increment(field_name, -amount);
    }

    pub fn add_to_array<T: Serialize>(&mut self, field_name: &str, items: &[T]) {
        let op = json!({
            "__op": "Add",
            "objects": items
        });
        self.fields.insert(field_name.to_string(), op);
    }

    pub fn add_unique_to_array<T: Serialize>(&mut self, field_name: &str, items: &[T]) {
        let op = json!({
            "__op": "AddUnique",
            "objects": items
        });
        self.fields.insert(field_name.to_string(), op);
    }

    pub fn remove_from_array<T: Serialize>(&mut self, field_name: &str, items: &[T]) {
        let op = json!({
            "__op": "Remove",
            "objects": items
        });
        self.fields.insert(field_name.to_string(), op);
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RetrievedParseObject {
    pub object_id: String,
    #[serde(deserialize_with = "deserialize_string_to_parse_date")]
    pub created_at: ParseDate,
    #[serde(deserialize_with = "deserialize_string_to_parse_date")]
    pub updated_at: ParseDate,
    #[serde(flatten)]
    pub fields: HashMap<String, Value>,
    #[serde(rename = "ACL")]
    pub acl: Option<ParseACL>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateObjectResponse {
    pub object_id: String,
    #[serde(deserialize_with = "deserialize_string_to_parse_date")]
    pub created_at: ParseDate,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateObjectResponse {
    #[serde(deserialize_with = "deserialize_string_to_parse_date")]
    pub updated_at: ParseDate,
}

impl ParseClient {
    pub async fn create_object<T: Serialize + Send + Sync>(
        &self,
        class_name: &str,
        data: &T,
    ) -> Result<CreateObjectResponse, ParseError> {
        if class_name.is_empty() {
            return Err(ParseError::InvalidInput(
                "Class name cannot be empty".to_string(),
            ));
        }
        if !class_name
            .chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c == '_')
        {
            return Err(ParseError::InvalidInput(
                "Invalid class name: must start with a letter or underscore.".to_string(),
            ));
        }
        if !class_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(ParseError::InvalidInput(
                "Invalid class name: can only contain letters, numbers, or underscores."
                    .to_string(),
            ));
        }

        let endpoint = format!("classes/{}", class_name);
        match self.post(&endpoint, data).await {
            Ok(res) => Ok(res),
            Err(e) => Err(e),
        }
    }

    pub async fn retrieve_object(
        &self,
        class_name: &str,
        object_id: &str,
    ) -> Result<RetrievedParseObject, ParseError> {
        if class_name.is_empty() {
            return Err(ParseError::InvalidInput(
                "Class name cannot be empty".to_string(),
            ));
        }
        if object_id.is_empty() {
            return Err(ParseError::InvalidInput(
                "Object ID cannot be empty".to_string(),
            ));
        }
        if !class_name
            .chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c == '_')
        {
            return Err(ParseError::InvalidInput(
                "Invalid class name: must start with a letter or underscore.".to_string(),
            ));
        }
        if !class_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(ParseError::InvalidInput(
                "Invalid class name: can only contain letters, numbers, or underscores."
                    .to_string(),
            ));
        }

        let endpoint = format!("classes/{}/{}", class_name, object_id);
        self.get(&endpoint).await
    }

    pub async fn update_object<T: Serialize + Send + Sync>(
        &self,
        class_name: &str,
        object_id: &str,
        data: &T,
    ) -> Result<UpdateObjectResponse, ParseError> {
        if class_name.is_empty() {
            return Err(ParseError::InvalidInput(
                "Class name cannot be empty".to_string(),
            ));
        }
        if object_id.is_empty() {
            return Err(ParseError::InvalidInput(
                "Object ID cannot be empty".to_string(),
            ));
        }
        if !class_name
            .chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c == '_')
        {
            return Err(ParseError::InvalidInput(
                "Invalid class name: must start with a letter or underscore.".to_string(),
            ));
        }
        if !class_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(ParseError::InvalidInput(
                "Invalid class name: can only contain letters, numbers, or underscores."
                    .to_string(),
            ));
        }

        let endpoint = format!("classes/{}/{}", class_name, object_id);
        self.put(&endpoint, data).await
    }

    pub async fn delete_object(&self, class_name: &str, object_id: &str) -> Result<(), ParseError> {
        if class_name.is_empty() {
            return Err(ParseError::InvalidInput(
                "Class name cannot be empty".to_string(),
            ));
        }
        if object_id.is_empty() {
            return Err(ParseError::InvalidInput(
                "Object ID cannot be empty".to_string(),
            ));
        }
        if !class_name
            .chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c == '_')
        {
            return Err(ParseError::InvalidInput(
                "Invalid class name: must start with a letter or underscore.".to_string(),
            ));
        }
        if !class_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(ParseError::InvalidInput(
                "Invalid class name: can only contain letters, numbers, or underscores."
                    .to_string(),
            ));
        }

        let endpoint = format!("classes/{}/{}", class_name, object_id);
        let response_value: Value = self.delete::<Value>(&endpoint).await?;

        if response_value.is_object()
            && response_value.as_object().is_some_and(|obj| obj.is_empty())
        {
            Ok(())
        } else {
            Err(ParseError::UnexpectedResponse(format!(
                "Expected empty JSON object {{}} for delete, got: {:?}",
                response_value
            )))
        }
    }
}
