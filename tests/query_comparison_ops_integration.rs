mod query_test_utils;

#[cfg(test)]
mod comparison_ops_tests {
    use super::query_test_utils::shared::*;
    use parse_rs::query::ParseQuery;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_query_greater_than() {
        let client = setup_client();
        let class_name = format!("TestComparisonOps_{}", Uuid::new_v4().simple());
        cleanup_test_class(&client, &class_name).await;

        let score1 = create_test_score(&client, &class_name, 50, "PlayerGt1", None, None)
            .await
            .unwrap();
        let score2 = create_test_score(&client, &class_name, 100, "PlayerGt2", None, None)
            .await
            .unwrap();
        let score3 = create_test_score(&client, &class_name, 150, "PlayerGt3", None, None)
            .await
            .unwrap();

        // Test greater_than
        let mut query = ParseQuery::new(&class_name);
        query.greater_than("score", 100);
        let results: Vec<GameScore> = query
            .find(&client)
            .await
            .expect("Query greater_than failed");
        assert_eq!(results.len(), 1, "Expected one player with score > 100");
        assert_eq!(results[0], score3);

        // Test with a value that should return multiple results
        let mut query_multi = ParseQuery::new(&class_name);
        query_multi.greater_than("score", 40);
        let results_multi: Vec<GameScore> = query_multi
            .find(&client)
            .await
            .expect("Query greater_than (multiple) failed");
        assert_eq!(
            results_multi.len(),
            3,
            "Expected three players with score > 40"
        );
        assert!(results_multi.contains(&score1));
        assert!(results_multi.contains(&score2));
        assert!(results_multi.contains(&score3));

        // Test with a value that should return no results
        let mut query_none = ParseQuery::new(&class_name);
        query_none.greater_than("score", 200);
        let results_none: Vec<GameScore> = query_none
            .find(&client)
            .await
            .expect("Query greater_than (none) failed");
        assert!(
            results_none.is_empty(),
            "Expected no players with score > 200"
        );

        cleanup_test_class(&client, &class_name).await;
    }

    #[tokio::test]
    async fn test_query_greater_than_or_equal_to() {
        let client = setup_client();
        let class_name = format!("TestComparisonOps_{}", Uuid::new_v4().simple());
        cleanup_test_class(&client, &class_name).await;

        let score1 = create_test_score(&client, &class_name, 50, "PlayerGte1", None, None)
            .await
            .unwrap();
        let score2 = create_test_score(&client, &class_name, 100, "PlayerGte2", None, None)
            .await
            .unwrap();
        let score3 = create_test_score(&client, &class_name, 150, "PlayerGte3", None, None)
            .await
            .unwrap();

        // Test greater_than_or_equal_to (matches one exactly, one greater)
        let mut query = ParseQuery::new(&class_name);
        query.greater_than_or_equal_to("score", 100);
        let results: Vec<GameScore> = query
            .find(&client)
            .await
            .expect("Query greater_than_or_equal_to failed");
        assert_eq!(results.len(), 2, "Expected two players with score >= 100");
        assert!(results.contains(&score2));
        assert!(results.contains(&score3));

        // Test with a value that should return all results (>= 50)
        let mut query_all = ParseQuery::new(&class_name);
        query_all.greater_than_or_equal_to("score", 50);
        let results_all: Vec<GameScore> = query_all
            .find(&client)
            .await
            .expect("Query gte (all) failed");
        assert_eq!(
            results_all.len(),
            3,
            "Expected three players with score >= 50"
        );
        assert!(results_all.contains(&score1));
        assert!(results_all.contains(&score2));
        assert!(results_all.contains(&score3));

        // Test with a value that should return no results (>= 200)
        let mut query_none = ParseQuery::new(&class_name);
        query_none.greater_than_or_equal_to("score", 200);
        let results_none: Vec<GameScore> = query_none
            .find(&client)
            .await
            .expect("Query gte (none) failed");
        assert!(
            results_none.is_empty(),
            "Expected no players with score >= 200"
        );

        cleanup_test_class(&client, &class_name).await;
    }

    #[tokio::test]
    async fn test_query_less_than() {
        let client = setup_client();
        let class_name = format!("TestComparisonOps_{}", Uuid::new_v4().simple());
        cleanup_test_class(&client, &class_name).await;

        let score1 = create_test_score(&client, &class_name, 50, "PlayerLt1", None, None)
            .await
            .unwrap();
        let score2 = create_test_score(&client, &class_name, 100, "PlayerLt2", None, None)
            .await
            .unwrap();
        let score3 = create_test_score(&client, &class_name, 150, "PlayerLt3", None, None)
            .await
            .unwrap();

        // Test less_than
        let mut query = ParseQuery::new(&class_name);
        query.less_than("score", 100);
        let results: Vec<GameScore> = query.find(&client).await.expect("Query less_than failed");
        assert_eq!(results.len(), 1, "Expected one player with score < 100");
        assert_eq!(results[0], score1);

        // Test with a value that should return multiple results (< 160)
        let mut query_multi = ParseQuery::new(&class_name);
        query_multi.less_than("score", 160);
        let results_multi: Vec<GameScore> = query_multi
            .find(&client)
            .await
            .expect("Query less_than (multiple) failed");
        assert_eq!(
            results_multi.len(),
            3,
            "Expected three players with score < 160"
        );
        assert!(results_multi.contains(&score1));
        assert!(results_multi.contains(&score2));
        assert!(results_multi.contains(&score3));

        // Test with a value that should return no results (< 50)
        let mut query_none = ParseQuery::new(&class_name);
        query_none.less_than("score", 50);
        let results_none: Vec<GameScore> = query_none
            .find(&client)
            .await
            .expect("Query less_than (none) failed");
        assert!(
            results_none.is_empty(),
            "Expected no players with score < 50"
        );

        cleanup_test_class(&client, &class_name).await;
    }

    #[tokio::test]
    async fn test_query_less_than_or_equal_to() {
        let client = setup_client();
        let class_name = format!("TestComparisonOps_{}", Uuid::new_v4().simple());
        cleanup_test_class(&client, &class_name).await;

        let score1 = create_test_score(&client, &class_name, 50, "PlayerLte1", None, None)
            .await
            .unwrap();
        let score2 = create_test_score(&client, &class_name, 100, "PlayerLte2", None, None)
            .await
            .unwrap();
        let score3 = create_test_score(&client, &class_name, 150, "PlayerLte3", None, None)
            .await
            .unwrap();

        // Test less_than_or_equal_to (matches one exactly, one less)
        let mut query = ParseQuery::new(&class_name);
        query.less_than_or_equal_to("score", 100);
        let results: Vec<GameScore> = query
            .find(&client)
            .await
            .expect("Query less_than_or_equal_to failed");
        assert_eq!(results.len(), 2, "Expected two players with score <= 100");
        assert!(results.contains(&score1));
        assert!(results.contains(&score2));

        // Test with a value that should return all results (<= 150)
        let mut query_all = ParseQuery::new(&class_name);
        query_all.less_than_or_equal_to("score", 150);
        let results_all: Vec<GameScore> = query_all
            .find(&client)
            .await
            .expect("Query lte (all) failed");
        assert_eq!(
            results_all.len(),
            3,
            "Expected three players with score <= 150"
        );
        assert!(results_all.contains(&score1));
        assert!(results_all.contains(&score2));
        assert!(results_all.contains(&score3));

        // Test with a value that should return no results (<= 40)
        let mut query_none = ParseQuery::new(&class_name);
        query_none.less_than_or_equal_to("score", 40);
        let results_none: Vec<GameScore> = query_none
            .find(&client)
            .await
            .expect("Query lte (none) failed");
        assert!(
            results_none.is_empty(),
            "Expected no players with score <= 40"
        );

        cleanup_test_class(&client, &class_name).await;
    }
}
