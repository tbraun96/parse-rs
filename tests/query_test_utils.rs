use dotenvy::dotenv;
use parse_rs::client::ParseClient as Parse;
use parse_rs::error::ParseError;
use parse_rs::query::ParseQuery;
use parse_rs::ParseObject;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Once;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

static INIT_LOGGER: Once = Once::new();

fn initialize_logger_once() {
    INIT_LOGGER.call_once(|| {
        env_logger::init();
    });
}

pub mod shared {
    use super::*;

    // Define a common structure for test objects if not already globally available
    // This TestObject can be used across different test files via `use crate::query_test_utils::TestObject;`
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)] // Added derives
    pub struct TestObject {
        #[serde(rename = "objectId", skip_serializing_if = "Option::is_none")]
        pub object_id: Option<String>,
        #[serde(rename = "createdAt", skip_serializing_if = "Option::is_none")]
        pub created_at: Option<String>,
        #[serde(rename = "updatedAt", skip_serializing_if = "Option::is_none")]
        pub updated_at: Option<String>,
        #[serde(flatten)] // Capture other fields dynamically
        pub fields: serde_json::Map<String, Value>,
    }

    // Generic helper to create any test object using serde_json::Value
    #[allow(dead_code)]
    pub async fn create_test_object(
        client: &Parse,
        class_name: &str,
        data: Value,
    ) -> Result<TestObject, ParseError> {
        let created_object_response = client.create_object(class_name, &data).await?;
        let object_id = created_object_response.object_id;

        let mut query = ParseQuery::new(class_name);
        query.equal_to("objectId", &object_id);

        let fetched_object: Option<TestObject> = query.first(client).await?;

        fetched_object.ok_or_else(|| {
            ParseError::Unknown(format!(
                "Failed to fetch created test object {} immediately after creation",
                object_id
            ))
        })
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    pub struct GameScore {
        #[serde(rename = "objectId", skip_serializing_if = "Option::is_none")]
        pub object_id: Option<String>,
        #[serde(rename = "createdAt", skip_serializing_if = "Option::is_none")]
        pub created_at: Option<String>,
        #[serde(rename = "updatedAt", skip_serializing_if = "Option::is_none")]
        pub updated_at: Option<String>,
        pub score: i32,
        pub player_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub cheat_mode: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub skills: Option<Vec<String>>,
    }

    // Helper to create a GameScore object for testing
    // This function remains useful for tests specifically dealing with GameScore structure
    #[allow(dead_code)]
    pub async fn create_test_score(
        client: &Parse,
        class_name: &str,
        score: i32,
        player_name: &str,
        cheat_mode: Option<bool>,
        skills: Option<Vec<String>>,
    ) -> Result<GameScore, ParseError> {
        let game_score_data = GameScore {
            object_id: None,
            created_at: None,
            updated_at: None,
            score,
            player_name: player_name.to_string(),
            cheat_mode,
            skills,
        };
        let created_object_response = client.create_object(class_name, &game_score_data).await?;

        let mut query = ParseQuery::new(class_name);
        query.equal_to("objectId", created_object_response.object_id.clone());
        let created_game_score: Option<GameScore> = query.first(client).await?;

        created_game_score.ok_or_else(|| {
            ParseError::Unknown(
                "Failed to fetch created test score immediately after creation".to_string(),
            )
        })
    }

    // Helper function to initialize Parse client from environment variables for tests NOT requiring master key
    #[allow(dead_code)]
    pub fn setup_client() -> Parse {
        initialize_logger_once();
        dotenv().ok();
        let app_id =
            std::env::var("PARSE_APP_ID").expect("PARSE_APP_ID not set for setup_client tests");
        let server_url = std::env::var("PARSE_SERVER_URL")
            .expect("PARSE_SERVER_URL not set for setup_client tests");
        let javascript_key = std::env::var("PARSE_JAVASCRIPT_KEY")
            .expect("PARSE_JAVASCRIPT_KEY not set for setup_client tests");
        Parse::new(&server_url, &app_id, Some(&javascript_key), None, None)
            .expect("Failed to create Parse client for setup_client tests")
    }

    // Helper function to initialize Parse client from environment variables for tests REQUIRING master key
    #[allow(dead_code)]
    pub fn setup_client_with_master_key() -> Parse {
        initialize_logger_once();
        dotenv().ok();
        let app_id =
            std::env::var("PARSE_APP_ID").expect("PARSE_APP_ID not set for master key tests");
        let server_url = std::env::var("PARSE_SERVER_URL")
            .expect("PARSE_SERVER_URL not set for master key tests");
        let master_key_str = std::env::var("PARSE_SERVER_MASTER_KEY")
            .expect("PARSE_SERVER_MASTER_KEY not set for master key tests");
        Parse::new(&server_url, &app_id, None, None, Some(&master_key_str))
            .expect("Failed to create Parse client with master key for tests")
    }

    #[allow(dead_code)]
    pub async fn cleanup_test_class(client: &Parse, class_name: &str) {
        let query = ParseQuery::new(class_name); // Create a ParseQuery

        match query.find::<ParseObject>(client).await {
            // Use query.find()
            Ok(objects) => {
                // Renamed response.results to objects for clarity
                for obj in objects {
                    if let Some(ref obj_id) = obj.object_id {
                        let _ = client.delete_object(class_name, obj_id.as_str()).await;
                        // Use obj_id.as_str()
                    }
                }
            }
            Err(e) => {
                eprintln!("Error querying class {} for cleanup: {:?}", class_name, e);
            }
        }
    }

    #[allow(dead_code)]
    pub fn generate_unique_classname(base: &str) -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        format!("{}_{}_{}", base, timestamp, Uuid::new_v4().simple()) // Changed to .simple()
    }
}
