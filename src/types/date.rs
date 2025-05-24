// src/types/date.rs

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone)]
pub struct ParseDate {
    pub iso: String,
}

impl ParseDate {
    pub fn new(iso_string: String) -> Self {
        ParseDate { iso: iso_string }
    }

    pub fn iso(&self) -> &str {
        &self.iso
    }
}

impl<'de> Deserialize<'de> for ParseDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        // Optionally, add validation here to ensure 's' is a valid ISO 8601 string
        // For now, we assume the server sends valid strings.
        Ok(ParseDate { iso: s })
    }
}

impl Serialize for ParseDate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.iso)
    }
}
