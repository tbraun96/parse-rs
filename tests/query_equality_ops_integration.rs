mod query_test_utils;

#[cfg(test)]
mod equality_ops_tests {
    use super::query_test_utils::shared::*; // Import all from shared
    use parse_rs::query::ParseQuery; // Specific to query tests
                                     // use parse_rs::error::ParseError; // Not strictly needed if only using ParseQuery methods that don't return Result directly in asserts
    use uuid::Uuid; // For unique class names

    #[tokio::test]
    async fn test_query_equal_to() {
        let client = setup_client();
        let class_name = format!("TestEqualityOps_{}", Uuid::new_v4().simple());
        cleanup_test_class(&client, &class_name).await;

        let score1 = create_test_score(&client, &class_name, 100, "PlayerEq1", Some(false), None)
            .await
            .unwrap();
        let score2 = create_test_score(
            &client,
            &class_name,
            200,
            "PlayerEq2",
            Some(true),
            Some(vec!["skillA".to_string()]),
        )
        .await
        .unwrap();
        let _score3 = create_test_score(&client, &class_name, 100, "PlayerEq3", Some(false), None)
            .await
            .unwrap();

        // Test equal_to for player_name
        let mut query_player = ParseQuery::new(&class_name);
        query_player.equal_to("player_name", "PlayerEq2");
        let results_player: Vec<GameScore> = query_player
            .find(&client)
            .await
            .expect("Query equal_to player_name failed");
        assert_eq!(results_player.len(), 1);
        assert_eq!(results_player[0], score2);

        // Test equal_to for score
        let mut query_score = ParseQuery::new(&class_name);
        query_score.equal_to("score", 100);
        let results_score: Vec<GameScore> = query_score
            .find(&client)
            .await
            .expect("Query equal_to score failed");
        assert_eq!(
            results_score.len(),
            2,
            "Expected two players with score 100"
        );
        assert!(results_score.contains(&score1));
        assert!(results_score.iter().any(|s| s.player_name == "PlayerEq3"));

        // Test equal_to for boolean
        let mut query_cheat = ParseQuery::new(&class_name);
        query_cheat.equal_to("cheat_mode", true);
        let results_cheat: Vec<GameScore> = query_cheat
            .find(&client)
            .await
            .expect("Query equal_to cheat_mode failed");
        assert_eq!(results_cheat.len(), 1);
        assert_eq!(results_cheat[0], score2);

        cleanup_test_class(&client, &class_name).await;
    }

    #[tokio::test]
    async fn test_query_not_equal_to() {
        let client = setup_client();
        let class_name = format!("TestEqualityOps_{}", Uuid::new_v4().simple());
        cleanup_test_class(&client, &class_name).await;

        let score1 = create_test_score(&client, &class_name, 100, "PlayerNeq1", Some(false), None)
            .await
            .unwrap();
        let score2 = create_test_score(&client, &class_name, 200, "PlayerNeq2", Some(true), None)
            .await
            .unwrap();
        let _score3 = create_test_score(&client, &class_name, 100, "PlayerNeq3", Some(false), None)
            .await
            .unwrap();

        // Test not_equal_to for player_name
        let mut query_player = ParseQuery::new(&class_name);
        query_player.not_equal_to("player_name", "PlayerNeq2");
        let results_player: Vec<GameScore> = query_player
            .find(&client)
            .await
            .expect("Query not_equal_to player_name failed");
        assert_eq!(
            results_player.len(),
            2,
            "Expected two players not named PlayerNeq2"
        );
        assert!(results_player.contains(&score1));
        assert!(results_player.iter().any(|s| s.player_name == "PlayerNeq3"));
        assert!(!results_player.contains(&score2));

        // Test not_equal_to for score
        let mut query_score = ParseQuery::new(&class_name);
        query_score.not_equal_to("score", 100);
        let results_score: Vec<GameScore> = query_score
            .find(&client)
            .await
            .expect("Query not_equal_to score failed");
        assert_eq!(
            results_score.len(),
            1,
            "Expected one player with score not 100"
        );
        assert_eq!(results_score[0], score2);

        // Test not_equal_to for boolean
        let mut query_cheat = ParseQuery::new(&class_name);
        query_cheat.not_equal_to("cheat_mode", true);
        let results_cheat: Vec<GameScore> = query_cheat
            .find(&client)
            .await
            .expect("Query not_equal_to cheat_mode failed");
        assert_eq!(
            results_cheat.len(),
            2,
            "Expected two players with cheat_mode not true (i.e. false or null)"
        );
        assert!(results_cheat.contains(&score1));
        assert!(results_cheat.iter().any(|s| s.player_name == "PlayerNeq3"));

        cleanup_test_class(&client, &class_name).await;
    }
}
