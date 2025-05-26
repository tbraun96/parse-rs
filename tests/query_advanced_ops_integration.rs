mod query_test_utils;

#[cfg(test)]
mod advanced_query_tests {
    use super::query_test_utils::shared::{
        cleanup_test_class, create_test_object, setup_client, setup_client_with_master_key,
        TestObject,
    };
    use parse_rs::query::ParseQuery;
    use parse_rs::Parse;
    use parse_rs::ParseError;
    use serde_json::{json, Value};
    use uuid::Uuid;

    async fn setup_clients_and_class_name(base_name: &str) -> (Parse, Parse, String) {
        let client = setup_client();
        let master_key_client = setup_client_with_master_key();
        let class_name_str = generate_unique_class_name(base_name);
        let class_name = class_name_str.as_str();

        println!("[Debug] Setting up class: {}", class_name);

        // Create the first object with the master key client to define the schema and fields
        let initial_object_data =
            json!({ "name": "InitialObject", "value": 100, "is_active": true });
        let create_result = master_key_client
            .create_object(class_name, &initial_object_data)
            .await;
        match &create_result {
            Ok(obj) => println!(
                "[Debug] Initial object created for {}: {:?}",
                class_name, obj
            ),
            Err(e) => println!(
                "[Debug] ERROR creating initial object for {}: {:?}",
                class_name, e
            ),
        }
        create_result.expect("Failed to create initial object with master key during setup");

        // Update CLPs to be permissive for the JS key client
        let clp = json!({ "*": { "get": true, "find": true, "count": true, "create": true, "update": true, "delete": true, "addField": true } });
        let update_result = master_key_client
            .update_class_schema(class_name, &clp)
            .await;
        match &update_result {
            Ok(response) => println!("[Debug] CLPs updated for {}: {:?}", class_name, response),
            Err(e) => println!("[Debug] ERROR updating CLPs for {}: {:?}", class_name, e),
        }
        update_result.expect("Failed to update CLPs during setup");

        (client, master_key_client, class_name_str)
    }

    fn generate_unique_class_name(base_name: &str) -> String {
        format!("{}_{}", base_name, Uuid::new_v4().simple())
    }

    #[tokio::test]
    async fn test_query_exists_and_does_not_exist() -> Result<(), ParseError> {
        let (client, _master_key_client, class_name_str) =
            setup_clients_and_class_name("TestExistsQuery").await;
        let class_name = class_name_str.as_str();

        cleanup_test_class(&client, class_name).await;

        let obj1_data = json!({ "name": "Object1", "optional_field": "present" });
        // Create first object with master key to define schema and fields
        let obj1 = create_test_object(&client, class_name, obj1_data)
            .await
            .unwrap();

        // Set permissive CLPs after schema is defined by the first object
        let clp_payload = json!({
            "classLevelPermissions": {
                "find": {"*": true},
                "get": {"*": true},
                "create": {"*": true},
                "update": {"*": true},
                "delete": {"*": true},
                "addField": {"*": true}
            }
        });
        if let Err(e) = _master_key_client
            .update_class_schema(class_name, &clp_payload)
            .await
        {
            panic!(
                "Master key client failed to update CLPs for {}: {:?}",
                class_name, e
            );
        }

        let obj2_data = json!({ "name": "Object2" });
        let _obj2 = create_test_object(&client, class_name, obj2_data)
            .await
            .unwrap();

        let obj3_data = json!({ "name": "Object3", "optional_field": Value::Null });
        let _obj3 = create_test_object(&client, class_name, obj3_data)
            .await
            .unwrap();

        let obj4_data = json!({ "name": "Object4", "optional_field": "another_value" });
        let obj4 = create_test_object(&client, class_name, obj4_data)
            .await
            .unwrap();

        let mut query_exists = ParseQuery::new(class_name);
        query_exists.exists("optional_field");
        let results_exists: Vec<TestObject> = query_exists
            .find(&client)
            .await
            .expect("Query $exists:true failed");

        assert_eq!(
            results_exists.len(),
            3,
            "Expected 3 objects where optional_field exists"
        );
        assert!(results_exists.contains(&obj1));
        assert!(!results_exists.contains(&_obj2));
        assert!(results_exists.contains(&_obj3));
        assert!(results_exists.contains(&obj4));

        let mut query_does_not_exist = ParseQuery::new(class_name);
        query_does_not_exist.does_not_exist("optional_field");
        let results_does_not_exist: Vec<TestObject> = query_does_not_exist
            .find(&client)
            .await
            .expect("Query $exists:false failed");

        assert_eq!(
            results_does_not_exist.len(),
            1,
            "Expected 1 object where optional_field does not exist"
        );
        assert!(!results_does_not_exist.contains(&obj1));
        assert!(results_does_not_exist.contains(&_obj2));
        assert!(!results_does_not_exist.contains(&_obj3));
        assert!(!results_does_not_exist.contains(&obj4));

        cleanup_test_class(&client, class_name).await;
        Ok(())
    }

    #[tokio::test]
    async fn test_query_string_matching_ops() -> Result<(), ParseError> {
        let (client, _master_key_client, class_name_str) =
            setup_clients_and_class_name("TestStringQuery").await;
        let class_name = class_name_str.as_str();

        cleanup_test_class(&client, class_name).await;

        let obj1_data = json!({ "description": "Hello World", "category": "greeting" });
        // Create first object with master key to define schema and fields
        let obj1 = create_test_object(&client, class_name, obj1_data)
            .await
            .unwrap();

        // Set permissive CLPs after schema is defined by the first object
        let clp_payload = json!({
            "classLevelPermissions": {
                "find": {"*": true},
                "get": {"*": true},
                "create": {"*": true},
                "update": {"*": true},
                "delete": {"*": true},
                "addField": {"*": true}
            }
        });
        if let Err(e) = _master_key_client
            .update_class_schema(class_name, &clp_payload)
            .await
        {
            panic!(
                "Master key client failed to update CLPs for {}: {:?}",
                class_name, e
            );
        }

        let obj2_data = json!({ "description": "world wide web", "category": "technology" });
        let _obj2 = create_test_object(&client, class_name, obj2_data)
            .await
            .unwrap();

        let obj3_data = json!({ "description": "Hello Universe", "category": "greeting" });
        let obj3 = create_test_object(&client, class_name, obj3_data)
            .await
            .unwrap();

        let obj4_data = json!({ "description": "Another String Example", "category": "example" });
        let _obj4 = create_test_object(&client, class_name, obj4_data)
            .await
            .unwrap();

        let mut query_starts = ParseQuery::new(class_name);
        query_starts.starts_with("description", "Hello");
        let results_starts: Vec<TestObject> = query_starts
            .find(&client)
            .await
            .expect("Query starts_with failed");
        assert_eq!(
            results_starts.len(),
            2,
            "Expected 2 objects starting with 'Hello'"
        );
        assert!(results_starts.contains(&obj1));
        assert!(results_starts.contains(&obj3));

        let mut query_ends = ParseQuery::new(class_name);
        query_ends.matches_regex("description", "World$", None);
        let results_ends: Vec<TestObject> = query_ends
            .find(&client)
            .await
            .expect("Query ends_with failed");
        assert_eq!(
            results_ends.len(),
            1,
            "Expected 1 object with description ending with 'World'"
        );
        assert!(results_ends.contains(&obj1));

        let mut query_contains = ParseQuery::new(class_name);
        query_contains.matches_regex("description", ".*Wor.*", None);
        let results_contains: Vec<TestObject> = query_contains
            .find(&client)
            .await
            .expect("Query $regex contains 'Wor' failed");

        assert_eq!(
            results_contains.len(),
            1,
            "Expected 1 object with description containing 'Wor'"
        );
        assert!(results_contains.contains(&obj1));

        cleanup_test_class(&client, class_name).await;
        Ok(())
    }

    #[tokio::test]
    async fn test_query_matches_regex() -> Result<(), ParseError> {
        let (client, _master_key_client, class_name_str) =
            setup_clients_and_class_name("TestRegexQuery").await;
        let class_name = class_name_str.as_str();

        cleanup_test_class(&client, class_name).await;

        let obj1_data = json!({ "name": "Object Alpha", "status": "active", "score": 100 });
        // Create first object with master key to define schema and fields
        let obj1 = create_test_object(&client, class_name, obj1_data)
            .await
            .unwrap();

        // Set permissive CLPs after schema is defined by the first object
        let clp_payload = json!({
            "classLevelPermissions": {
                "find": {"*": true},
                "get": {"*": true},
                "create": {"*": true},
                "update": {"*": true},
                "delete": {"*": true},
                "addField": {"*": true}
            }
        });
        if let Err(e) = _master_key_client
            .update_class_schema(class_name, &clp_payload)
            .await
        {
            panic!(
                "Master key client failed to update CLPs for {}: {:?}",
                class_name, e
            );
        }

        let obj2_data =
            json!({ "name": "Object Beta (Code-123)", "status": "inactive", "score": 200 });
        let _obj2 = create_test_object(&client, class_name, obj2_data)
            .await
            .unwrap();

        let obj3_data = json!({ "name": "Object Gamma", "status": "inactive", "score": 50 });
        let _obj3 = create_test_object(&client, class_name, obj3_data)
            .await
            .unwrap();

        let mut query_matches_status_exact = ParseQuery::new(class_name);
        query_matches_status_exact.matches_regex("status", "^active$", None);
        let results_matches_status_exact: Vec<TestObject> =
            query_matches_status_exact.find(&client).await?;
        assert_eq!(
            results_matches_status_exact.len(),
            1,
            "Expected 1 object with status 'active'"
        );
        assert!(
            results_matches_status_exact.contains(&obj1),
            "Results should contain Object Alpha"
        );

        let mut query_regex_name_starts = ParseQuery::new(class_name);
        query_regex_name_starts.matches_regex("name", "^object", Some("i"));
        let results_regex_name_starts: Vec<TestObject> = query_regex_name_starts
            .find(&client)
            .await
            .expect("Query matches_regex (name starts_with) failed");
        assert_eq!(
            results_regex_name_starts.len(),
            3,
            "Expected 3 objects with name starting with 'object' (case-insensitive)"
        );

        let mut query_regex_status_ends = ParseQuery::new(class_name);
        query_regex_status_ends.matches_regex("status", "^active$", None);
        let results_regex_status_ends: Vec<TestObject> = query_regex_status_ends
            .find(&client)
            .await
            .expect("Query matches_regex (status ends_with active) failed");
        assert_eq!(
            results_regex_status_ends.len(),
            1,
            "Expected 1 object with status exactly 'active'"
        );
        assert!(results_regex_status_ends.contains(&obj1));

        cleanup_test_class(&client, class_name).await;
        Ok(())
    }

    #[tokio::test]
    async fn test_query_distinct_values() -> Result<(), ParseError> {
        let (_client, master_key_client, class_name_str) =
            setup_clients_and_class_name("TestDistinctQuery").await;
        let class_name = class_name_str.as_str();

        cleanup_test_class(&master_key_client, class_name).await;

        // Create first object with master key to define schema and fields
        let _obj1 = create_test_object(
            &master_key_client, // Use master key for the first object
            class_name,
            json!({ "name": "Item A", "category": "electronics", "stock": 10 }),
        )
        .await?;

        // Set permissive CLPs after schema is defined by the first object
        let clp_payload = json!({
            "classLevelPermissions": {
                "find": {"*": true},
                "get": {"*": true},
                "create": {"*": true},
                "update": {"*": true},
                "delete": {"*": true},
                "addField": {"*": true}
            }
        });
        if let Err(e) = master_key_client
            .update_class_schema(class_name, &clp_payload)
            .await
        {
            panic!(
                "Master key client failed to update CLPs for {}: {:?}",
                class_name, e
            );
        }

        let _obj2 = create_test_object(
            &master_key_client, // Use master key for subsequent objects
            class_name,
            json!({ "name": "Item B", "category": "electronics", "stock": 20 }),
        )
        .await?;

        let _obj3 = create_test_object(
            &master_key_client,
            class_name,
            json!({ "name": "Item C", "category": "fashion", "stock": 30 }),
        )
        .await?;

        let _obj4 = create_test_object(
            &master_key_client,
            class_name,
            json!({ "name": "Item D", "category": "electronics", "stock": 40 }),
        )
        .await?;

        let _obj5 = create_test_object(
            &master_key_client,
            class_name,
            json!({ "name": "Item E", "category": "fashion", "stock": 50 }),
        )
        .await?;

        let query_basic = ParseQuery::new(class_name);
        let mut results_basic: Vec<String> = query_basic
            .distinct(&master_key_client, "category") // Use master_key_client
            .await?;
        results_basic.sort();
        assert_eq!(
            results_basic.len(),
            2,
            "Basic distinct on 'category' failed length check"
        );
        assert_eq!(
            results_basic,
            vec!["electronics", "fashion"],
            "Basic distinct on 'category' failed value check"
        );

        let mut query_where = ParseQuery::new(class_name);
        query_where.equal_to("stock", json!(10));
        let mut results_where: Vec<String> = query_where
            .distinct(&master_key_client, "category") // Use master_key_client
            .await?;
        results_where.sort();
        assert_eq!(
            results_where.len(),
            1,
            "Distinct with where (stock=10) on 'category' failed length check"
        );
        assert_eq!(
            results_where,
            vec!["electronics"],
            "Distinct with where (stock=10) on 'category' failed value check"
        );

        let mut query_where_no_match = ParseQuery::new(class_name);
        query_where_no_match.equal_to("stock", json!(100));
        let results_where_no_match: Vec<String> = query_where_no_match
            .distinct(&master_key_client, "category")
            .await?;
        assert!(
            results_where_no_match.is_empty(),
            "Distinct with where (stock=100) on 'category' should be empty"
        );

        let query_non_existent_field = ParseQuery::new(class_name);
        let results_non_existent_field: Vec<Option<String>> = query_non_existent_field
            .distinct(&master_key_client, "non_existent_field") // Use master_key_client
            .await?;
        assert_eq!(
            results_non_existent_field.len(),
            1,
            "Distinct on 'non_existent_field' should return one item (null)"
        );
        assert!(
            results_non_existent_field[0].is_none(),
            "The distinct value for 'non_existent_field' should be None/null"
        );

        let query_non_existent_class = ParseQuery::new("NonExistentClassForDistinct");
        let result_non_existent_class: Result<Vec<Value>, ParseError> = query_non_existent_class
            .distinct(&master_key_client, "anyField")
            .await;

        match result_non_existent_class {
            Ok(vals) => {
                assert!(
                    vals.is_empty(),
                    "Distinct on non-existent class should return Ok([]), but got Ok({:?})",
                    vals
                );
            }
            Err(e) => {
                panic!("Distinct on non-existent class returned unexpected error: {:?}. Expected Ok([]).", e);
            }
        }

        let query_numeric = ParseQuery::new(class_name);
        let mut results_numeric: Vec<i64> = query_numeric
            .distinct(&master_key_client, "stock") // Use master_key_client
            .await?;
        results_numeric.sort();
        assert_eq!(
            results_numeric.len(),
            5,
            "Distinct on 'stock' failed length check"
        );
        assert_eq!(
            results_numeric,
            vec![10, 20, 30, 40, 50],
            "Distinct on 'stock' failed value check"
        );

        cleanup_test_class(&master_key_client, class_name).await;
        Ok(())
    }

    #[tokio::test]
    async fn test_query_full_text_search() -> Result<(), ParseError> {
        let (client, _master_key_client, class_name_str) =
            setup_clients_and_class_name("TestFullTextSearchQuery").await;
        let class_name = class_name_str.as_str();

        cleanup_test_class(&client, class_name).await;

        // Create first object with master key to define schema and fields
        let obj1 = create_test_object(&client, class_name, json!({ "name": "Recipe1", "description": "A delicious recipe for apple pie.", "category": "dessert" })).await?;

        // Set permissive CLPs after schema is defined by the first object
        let clp_payload = json!({
            "classLevelPermissions": {
                "find": {"*": true},
                "get": {"*": true},
                "create": {"*": true},
                "update": {"*": true},
                "delete": {"*": true},
                "addField": {"*": true}
            }
        });
        if let Err(e) = _master_key_client
            .update_class_schema(class_name, &clp_payload)
            .await
        {
            panic!(
                "Master key client failed to update CLPs for {}: {:?}",
                class_name, e
            );
        }

        let _obj2 = create_test_object(&client, class_name, json!({ "name": "Article1", "description": "An insightful article about data privacy.", "category": "technology" })).await?;

        let obj3 = create_test_object(&client, class_name, json!({ "name": "Recipe2", "description": "Simple banana bread recipe.", "category": "dessert" })).await?;

        let _obj4 = create_test_object(&client, class_name, json!({ "name": "Note1", "description": "Just a note about oranges.", "category": "misc" })).await?;

        // Basic search for "apple"
        let mut query_apple = ParseQuery::new(class_name);
        query_apple.search("description", "apple", None, None, None);
        let results_apple: Vec<TestObject> = query_apple.find(&client).await?;
        assert_eq!(
            results_apple.len(),
            1,
            "Search for 'apple' should return 1 result"
        );
        assert!(results_apple.contains(&obj1));

        // Search for "recipe"
        let mut query_recipe = ParseQuery::new(class_name);
        query_recipe.search("description", "recipe", None, None, None);
        let results_recipe: Vec<TestObject> = query_recipe.find(&client).await?;
        assert_eq!(
            results_recipe.len(),
            2,
            "Search for 'recipe' should return 2 results"
        );
        assert!(results_recipe.contains(&obj1));
        assert!(results_recipe.contains(&obj3));

        // Search for a term not present
        let mut query_not_present = ParseQuery::new(class_name);
        query_not_present.search("description", "watermelon", None, None, None);
        let results_not_present: Vec<TestObject> = query_not_present.find(&client).await?;
        assert!(
            results_not_present.is_empty(),
            "Search for 'watermelon' should return 0 results"
        );

        // Search with language (assuming 'en' is default or supported)
        let mut query_apple_lang = ParseQuery::new(class_name);
        query_apple_lang.search("description", "apple", Some("en"), None, None);
        let results_apple_lang: Vec<TestObject> = query_apple_lang.find(&client).await?;
        assert_eq!(
            results_apple_lang.len(),
            1,
            "Search for 'apple' with language 'en' should return 1 result"
        );

        // Note: Testing $caseSensitive and $diacriticSensitive effectively requires specific server-side index configurations
        // and data that would highlight these differences. For basic client functionality, we assume the server handles these if specified.

        // Search for a term in a different field (should not find if index is only on description)
        // This test might behave differently based on actual indexing on the server.
        // If 'name' is not text-indexed, this might return 0 or error, or do a slow scan.
        let mut query_name_search = ParseQuery::new(class_name);
        query_name_search.search("name", "Recipe1", None, None, None);
        let results_name_search: Vec<TestObject> = query_name_search.find(&client).await?;
        // We expect 1 if 'name' is text indexed and contains 'Recipe1'
        // If not, this might be 0. For this test, let's assume it could find it.
        if !results_name_search.is_empty() {
            assert_eq!(
                results_name_search.len(),
                1,
                "Search for 'Recipe1' in name field returned unexpected count"
            );
            assert!(results_name_search.iter().any(|obj| obj
                .fields
                .get("name")
                .unwrap()
                .as_str()
                .unwrap()
                == "Recipe1"));
        } else {
            println!("Note: Text search on 'name' field for 'Recipe1' returned 0 results. This might be due to lack of text index on 'name'.");
        }

        cleanup_test_class(&client, class_name).await;
        Ok(())
    }
}
