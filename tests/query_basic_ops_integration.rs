mod query_test_utils;

#[cfg(test)]
mod basic_ops_tests {
    use super::query_test_utils::shared::*; // Import all from shared
    use parse_rs::error::ParseError; // For error checking
    use parse_rs::query::ParseQuery; // Specific to query tests
    use uuid::Uuid; // For unique class names

    #[tokio::test]
    async fn test_query_initial_setup_and_find_empty() {
        let client = setup_client();
        let class_name = format!("TestBasicOps_{}", Uuid::new_v4().simple());
        cleanup_test_class(&client, &class_name).await; // Ensure class is empty

        let query = ParseQuery::new(&class_name);
        let results: Vec<GameScore> = query.find(&client).await.expect("Query find failed");
        assert!(results.is_empty(), "Expected no results from empty class");

        // Test count on empty class
        let count = query.count(&client).await.expect("Query count failed");
        assert_eq!(count, 0, "Expected count of 0 for empty class");
    }

    #[tokio::test]
    async fn test_query_create_and_find_one() {
        let client = setup_client();
        let class_name = format!("TestBasicOps_{}", Uuid::new_v4().simple());
        cleanup_test_class(&client, &class_name).await;

        let player_name = "PlayerOneFind";
        let score_val = 100;

        let created_score = create_test_score(
            &client,
            &class_name,
            score_val,
            player_name,
            Some(false),
            None,
        )
        .await
        .expect("Failed to create test score");

        assert_eq!(created_score.player_name, player_name);
        assert_eq!(created_score.score, score_val);
        assert!(
            created_score.object_id.is_some(),
            "Created score should have an objectId"
        );

        // Now try to find it specifically
        let mut query = ParseQuery::new(&class_name);
        query.equal_to("objectId", created_score.object_id.as_ref().unwrap());

        let results: Vec<GameScore> = query
            .find(&client)
            .await
            .expect("Query find by objectId failed");
        assert_eq!(
            results.len(),
            1,
            "Expected to find exactly one object by its ID"
        );
        assert_eq!(
            results[0], created_score,
            "Found object does not match created object"
        );

        // Test first() as well
        let first_result: Option<GameScore> = query
            .first(&client)
            .await
            .expect("Query first by objectId failed");
        assert!(
            first_result.is_some(),
            "Expected first() to return Some for existing objectId"
        );
        assert_eq!(
            first_result.unwrap(),
            created_score,
            "First object does not match created object"
        );

        cleanup_test_class(&client, &class_name).await;
    }

    #[tokio::test]
    async fn test_query_get_object() {
        let client = setup_client();
        let class_name = format!("TestBasicOps_{}", Uuid::new_v4().simple());
        cleanup_test_class(&client, &class_name).await;

        let created_score = create_test_score(&client, &class_name, 500, "PlayerGet", None, None)
            .await
            .unwrap();
        let object_id = created_score
            .object_id
            .as_ref()
            .expect("Created score missing objectId for get test")
            .clone();

        let query = ParseQuery::new(&class_name);
        let fetched_score: GameScore = query
            .get(&object_id, &client)
            .await
            .expect("Query get failed");

        assert_eq!(
            fetched_score, created_score,
            "Fetched object via get() does not match created object"
        );

        // Test get with a non-existent ID
        let query_non_existent = ParseQuery::new(&class_name);
        let non_existent_id = "thisIdShouldNotExist";
        let get_result_non_existent: Result<GameScore, _> =
            query_non_existent.get(non_existent_id, &client).await;
        assert!(
            matches!(get_result_non_existent, Err(ParseError::ObjectNotFound(_))),
            "Expected ObjectNotFound for non-existent object, got {:?}",
            get_result_non_existent
        );

        cleanup_test_class(&client, &class_name).await;
    }
}
