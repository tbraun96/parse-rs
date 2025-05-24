// src/object.rs

use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)] // Add Serialize, Deserialize later as needed
pub struct ParseObject {
    pub class_name: String,
    pub object_id: Option<String>,
    pub created_at: Option<String>, // Or a proper ParseDate type
    pub updated_at: Option<String>, // Or a proper ParseDate type
    pub fields: HashMap<String, Value>,
}
