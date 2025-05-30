// tests/analytics_integration.rs
use parse_rs::error::ParseError;
use serde_json::json;
use uuid::Uuid;

mod query_test_utils;
use query_test_utils::shared::setup_client_with_master_key;

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_track_event_no_dimensions() {
        let client = setup_client_with_master_key(); // Analytics might need JS or REST key too
        let event_name = format!("TestEvent_{}", Uuid::new_v4().simple());

        let result = client.track_event(&event_name, None).await;
        assert!(
            result.is_ok(),
            "Failed to track event with no dimensions: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_track_event_with_dimensions() {
        let client = setup_client_with_master_key();
        let event_name = format!("TestEventDim_{}", Uuid::new_v4().simple());
        let dimensions = json!({
            "category": "test_category",
            "value": 123,
            "source": "integration_test"
        });

        let result = client.track_event(&event_name, Some(dimensions)).await;
        assert!(
            result.is_ok(),
            "Failed to track event with dimensions: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_track_event_empty_name_fails() {
        let client = setup_client_with_master_key();
        let event_name = "";

        let result = client.track_event(event_name, None).await;
        assert!(
            result.is_err(),
            "Tracking event with empty name should fail"
        );

        match result.err().unwrap() {
            ParseError::InvalidInput(msg) => {
                assert_eq!(msg, "Event name cannot be empty.");
            }
            _ => panic!("Expected InvalidInput error for empty event name"),
        }
    }
}
