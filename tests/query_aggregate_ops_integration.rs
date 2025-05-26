//! Integration tests for aggregate query operations.

mod query_test_utils;
use crate::query_test_utils::shared::setup_client_with_master_key;
use crate::query_test_utils::shared::{cleanup_test_class, create_test_object};
use dotenvy::dotenv;
use parse_rs::client::ParseClient;
use parse_rs::error::ParseError;
use parse_rs::query::ParseQuery;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

#[cfg(test)]
mod aggregate_query_tests {
    use super::*;

    #[derive(Deserialize, Debug, PartialEq)]
    struct SumResult {
        #[serde(rename = "objectId")]
        object_id: Option<String>,
        #[serde(rename = "totalScore")]
        total_score: f64,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct AvgResult {
        #[serde(rename = "objectId")]
        object_id: Option<String>,
        #[serde(rename = "avgScore")]
        avg_score: f64,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct GroupedResult {
        #[serde(rename = "objectId")]
        player_name_group: String,
        #[serde(rename = "totalPlayerScore")]
        total_player_score: f64,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct ProjectedHighScore {
        #[serde(rename = "playerName")]
        player_name: String,
        #[serde(rename = "highScore")]
        high_score: i32,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    struct TestScoreObject {
        #[serde(rename = "objectId", skip_serializing_if = "Option::is_none")]
        object_id: Option<String>,
        #[serde(rename = "playerName")]
        player_name: String,
        score: i32,
    }

    async fn setup_test_data(client: &ParseClient, class_name: &str) {
        let scores_data = vec![
            TestScoreObject {
                object_id: None,
                player_name: "PlayerA".to_string(),
                score: 100,
            },
            TestScoreObject {
                object_id: None,
                player_name: "PlayerB".to_string(),
                score: 200,
            },
            TestScoreObject {
                object_id: None,
                player_name: "PlayerA".to_string(),
                score: 150,
            },
            TestScoreObject {
                object_id: None,
                player_name: "PlayerC".to_string(),
                score: 50,
            },
            TestScoreObject {
                object_id: None,
                player_name: "PlayerB".to_string(),
                score: 250,
            },
        ];

        for data in scores_data {
            let json_data = serde_json::to_value(&data).unwrap();
            create_test_object(client, class_name, json_data)
                .await
                .unwrap();
        }
    }

    #[tokio::test]
    async fn test_aggregate_sum_and_avg_scores() {
        dotenv().ok();
        let client = setup_client_with_master_key();
        let class_name = format!("TestScores_{}", Uuid::new_v4().simple());

        setup_test_data(&client, &class_name).await;

        let pipeline_sum = vec![json!({
            "$group": {
                "_id": null,
                "totalScore": { "$sum": "$score" }
            }
        })];

        let query_sum = ParseQuery::new(&class_name);
        let results_sum: Vec<SumResult> = query_sum.aggregate(pipeline_sum, &client).await.unwrap();

        assert_eq!(results_sum.len(), 1);
        assert_eq!(results_sum[0].total_score, 750.0);

        let pipeline_avg = vec![json!({
            "$group": {
                "_id": null,
                "avgScore": { "$avg": "$score" }
            }
        })];

        let query_avg = ParseQuery::new(&class_name);
        let results_avg: Vec<AvgResult> = query_avg.aggregate(pipeline_avg, &client).await.unwrap();

        assert_eq!(results_avg.len(), 1);
        assert_eq!(results_avg[0].avg_score, 150.0);

        cleanup_test_class(&client, &class_name).await;
    }

    #[tokio::test]
    async fn test_aggregate_group_by_player_and_sum_scores() -> Result<(), ParseError> {
        dotenv().ok();
        let client = setup_client_with_master_key();
        let class_name = format!("TestScores_{}", Uuid::new_v4().simple());

        setup_test_data(&client, &class_name).await;

        let pipeline = vec![
            json!({
                "$group": {
                    "_id": "$playerName",
                    "totalPlayerScore": { "$sum": "$score" }
                }
            }),
            json!({
                "$sort": { "_id": 1 }
            }),
        ];

        let query = ParseQuery::new(&class_name);
        let results: Vec<GroupedResult> = query.aggregate(pipeline, &client).await?;

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].player_name_group, "PlayerA");
        assert_eq!(results[0].total_player_score, 250.0);
        assert_eq!(results[1].player_name_group, "PlayerB");
        assert_eq!(results[1].total_player_score, 450.0);
        assert_eq!(results[2].player_name_group, "PlayerC");
        assert_eq!(results[2].total_player_score, 50.0);

        cleanup_test_class(&client, &class_name).await;
        Ok(())
    }

    #[tokio::test]
    async fn test_aggregate_with_match_and_project() -> Result<(), ParseError> {
        dotenv().ok();
        let client = setup_client_with_master_key();
        let class_name = format!("TestScores_{}", Uuid::new_v4().simple());

        setup_test_data(&client, &class_name).await;

        let pipeline = vec![
            json!({
                "$match": { "score": { "$gt": 100 } }
            }),
            json!({
                "$project": {
                    "_id": 0,
                    "playerName": 1,
                    "highScore": "$score"
                }
            }),
            json!({
                "$sort": { "playerName": 1, "highScore": 1 }
            }),
        ];

        let query = ParseQuery::new(&class_name);
        let results: Vec<ProjectedHighScore> = query.aggregate(pipeline, &client).await?;

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].player_name, "PlayerA");
        assert_eq!(results[0].high_score, 150);
        assert_eq!(results[1].player_name, "PlayerB");
        assert_eq!(results[1].high_score, 200);
        assert_eq!(results[2].player_name, "PlayerB");
        assert_eq!(results[2].high_score, 250);

        cleanup_test_class(&client, &class_name).await;
        Ok(())
    }

    #[tokio::test]
    async fn test_aggregate_empty_results() -> Result<(), ParseError> {
        dotenv().ok();
        let client = setup_client_with_master_key();
        let class_name = format!("TestScoresEmpty_{}", Uuid::new_v4().simple());

        let pipeline = vec![json!({
            "$group": {
                "_id": null,
                "totalScore": { "$sum": "$score" }
            }
        })];

        let query = ParseQuery::new(&class_name);
        let results: Vec<SumResult> = query.aggregate(pipeline, &client).await?;

        assert!(
            results.is_empty(),
            "Expected empty results for an empty class or non-matching query"
        );

        cleanup_test_class(&client, &class_name).await;
        Ok(())
    }

    #[tokio::test]
    async fn test_aggregate_invalid_pipeline() {
        dotenv().ok();
        let client = setup_client_with_master_key();
        let class_name = format!("TestScoresInvalid_{}", Uuid::new_v4().simple());

        setup_test_data(&client, &class_name).await;

        let pipeline = vec![json!({
            "$invalidOperator": { "field": "$score" }
        })];

        let query = ParseQuery::new(&class_name);
        let result: Result<Vec<Value>, ParseError> = query.aggregate(pipeline, &client).await;

        println!("{:?}", result);
        assert!(
            matches!(result, Err(ParseError::InvalidQuery(_))),
            "Expected InvalidQuery for invalid pipeline, got {:?}",
            result
        );

        cleanup_test_class(&client, &class_name).await;
    }

    #[tokio::test]
    async fn test_aggregate_on_non_existent_class() -> Result<(), ParseError> {
        dotenv().ok();
        let client = setup_client_with_master_key();
        let class_name = format!("NonExistentClass_{}", Uuid::new_v4().simple());

        let pipeline = vec![json!({
            "$group": {
                "_id": null,
                "totalScore": { "$sum": "$score" }
            }
        })];

        let query = ParseQuery::new(&class_name);
        let results: Vec<SumResult> = query.aggregate(pipeline, &client).await?;

        assert!(
            results.is_empty(),
            "Expected no results from non-existent class"
        );

        Ok(())
    }
}
