// src/types/date.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

    /// Creates a new ParseDate representing the current time in UTC.
    pub fn now() -> Self {
        Self::from_datetime(Utc::now())
    }

    /// Creates a new ParseDate from a chrono::DateTime<Utc> object.
    pub fn from_datetime(dt: DateTime<Utc>) -> Self {
        ParseDate {
            __type: "Date".to_string(),
            iso: dt.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        }
    }

    /// Attempts to parse the ISO string into a chrono::DateTime<Utc> object.
    pub fn to_datetime(&self) -> Result<DateTime<Utc>, chrono::ParseError> {
        DateTime::parse_from_rfc3339(&self.iso).map(|dt| dt.with_timezone(&Utc))
    }
}
