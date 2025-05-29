// src/geopoint.rs

use serde::{Deserialize, Serialize};

/// Represents a geographical point.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ParseGeoPoint {
    #[serde(rename = "__type")]
    type_field: String, // Should always be "GeoPoint"
    pub latitude: f64,
    pub longitude: f64,
}

impl ParseGeoPoint {
    /// Creates a new `ParseGeoPoint`.
    ///
    /// # Panics
    /// Panics if latitude is not between -90 and 90, or longitude is not between -180 and 180.
    pub fn new(latitude: f64, longitude: f64) -> Self {
        if !(-90.0..=90.0).contains(&latitude) {
            panic!("Latitude must be between -90 and 90 degrees.");
        }
        if !(-180.0..=180.0).contains(&longitude) {
            panic!("Longitude must be between -180 and 180 degrees.");
        }
        ParseGeoPoint {
            type_field: "GeoPoint".to_string(),
            latitude,
            longitude,
        }
    }
}
