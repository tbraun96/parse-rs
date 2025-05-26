mod query_test_utils;

#[cfg(test)]
mod relational_query_tests {
    use super::query_test_utils::shared::*; // Corrected: use shared module
    use parse_rs::client::ParseClient; // Add this import
    use parse_rs::query::ParseQuery;
    use parse_rs::ParseError;
    use serde::{Deserialize, Serialize};
    use serde_json::{json, Value};
    use uuid::Uuid;

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    struct Author {
        #[serde(rename = "objectId", skip_serializing_if = "Option::is_none")]
        object_id: Option<String>,
        name: String,
        birth_year: Option<i32>,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    struct Book {
        #[serde(rename = "objectId", skip_serializing_if = "Option::is_none")]
        object_id: Option<String>,
        title: String,
        author_ptr: Value, // Will be Pointer or full Author object
        pages: Option<i32>,
    }

    async fn setup_author_book_data(
        client: &ParseClient,
        author_class_name: &str,
        book_class_name: &str,
    ) -> Result<(Author, Book), ParseError> {
        let author_data = json!({ "name": "Test Author", "birth_year": 1980 });
        let created_author_response = client
            .create_object(author_class_name, &author_data)
            .await?;
        let author_id = created_author_response.object_id;

        let author_obj = Author {
            object_id: Some(author_id.clone()), // Corrected: Wrap in Some()
            name: "Test Author".to_string(),
            birth_year: Some(1980),
        };

        let book_data = json!({
            "title": "Test Book About Pointers",
            "author_ptr": {
                "__type": "Pointer",
                "className": author_class_name,
                "objectId": author_id
            },
            "pages": 300
        });
        let created_book_response = client.create_object(book_class_name, &book_data).await?;
        let book_id = created_book_response.object_id;

        let book_obj = Book {
            object_id: Some(book_id.clone()), // Corrected: Wrap in Some() and clone if needed elsewhere
            title: "Test Book About Pointers".to_string(),
            author_ptr: book_data["author_ptr"].clone(), // Initially a pointer stub
            pages: Some(300),
        };
        Ok((author_obj, book_obj))
    }

    #[tokio::test]
    async fn test_query_include_pointer() -> Result<(), ParseError> {
        let client = setup_client();
        let author_class = format!("Author_{}", Uuid::new_v4().simple());
        let book_class = format!("Book_{}", Uuid::new_v4().simple());

        cleanup_test_class(&client, &author_class).await;
        cleanup_test_class(&client, &book_class).await;

        let (expected_author, _expected_book) =
            setup_author_book_data(&client, &author_class, &book_class).await?;

        // Test with include
        let mut query_include = ParseQuery::new(&book_class);
        query_include.include(&["author_ptr"]);
        let results_include: Vec<Book> = query_include
            .find(&client)
            .await
            .expect("Query with include failed");

        assert_eq!(results_include.len(), 1, "Expected one book with include");
        let book_with_include = results_include.first().unwrap();

        assert_eq!(book_with_include.title, "Test Book About Pointers");
        match book_with_include.author_ptr.as_object() {
            Some(author_map) => {
                assert_ne!(
                    author_map.get("__type").and_then(Value::as_str),
                    Some("Pointer"),
                    "author_ptr should be a full object, not a pointer stub"
                );
                assert_eq!(
                    author_map.get("name").and_then(Value::as_str),
                    Some(expected_author.name.as_str())
                );
                assert_eq!(
                    author_map.get("birth_year").and_then(Value::as_i64),
                    expected_author.birth_year.map(|y| y as i64)
                );
            }
            None => panic!("author_ptr was not an object when included"),
        }

        // Test without include
        let query_no_include = ParseQuery::new(&book_class);
        let results_no_include: Vec<Book> = query_no_include
            .find(&client)
            .await
            .expect("Query without include failed");

        assert_eq!(
            results_no_include.len(),
            1,
            "Expected one book without include"
        );
        let book_without_include = results_no_include.first().unwrap();

        assert_eq!(book_without_include.title, "Test Book About Pointers");
        match book_without_include.author_ptr.as_object() {
            Some(author_map) => {
                assert_eq!(
                    author_map.get("__type").and_then(Value::as_str),
                    Some("Pointer"),
                    "author_ptr should be a pointer stub"
                );
                assert_eq!(
                    author_map.get("className").and_then(Value::as_str),
                    Some(author_class.as_str())
                );
                assert_eq!(
                    author_map.get("objectId").and_then(Value::as_str),
                    Some(expected_author.object_id.as_ref().unwrap().as_str())
                );
            }
            None => panic!("author_ptr was not an object (pointer stub) when not included"),
        }

        cleanup_test_class(&client, &book_class).await;
        cleanup_test_class(&client, &author_class).await;

        Ok(())
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    struct SelectItem {
        #[serde(rename = "objectId", skip_serializing_if = "Option::is_none")]
        object_id: Option<String>,
        #[serde(rename = "createdAt", skip_serializing_if = "Option::is_none")]
        created_at: Option<String>,
        #[serde(rename = "updatedAt", skip_serializing_if = "Option::is_none")]
        updated_at: Option<String>,
        name: String,
        value: Option<i32>, // Expect this to be Some when not selected, None when selected out
        category: Option<String>, // Expect this to be Some when not selected, None when selected out
        is_active: Option<bool>, // Expect this to be Some when not selected, None when selected out
    }

    // Helper to deserialize into a generic Value to check for field presence/absence
    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    struct GenericItem {
        #[serde(rename = "objectId")]
        object_id: String,
        #[serde(flatten)]
        fields: serde_json::Map<String, Value>,
    }

    #[tokio::test]
    async fn test_query_select_keys() -> Result<(), ParseError> {
        let client = setup_client();
        let class_name = format!("SelectTestItem_{}", Uuid::new_v4().simple());
        cleanup_test_class(&client, &class_name).await;

        let item1_data = json!({
            "name": "Item Alpha",
            "value": 100,
            "category": "A",
            "is_active": true
        });
        let _item1 = create_test_object(&client, &class_name, item1_data.clone()).await?;

        let item2_data = json!({
            "name": "Item Beta",
            "value": 200,
            "category": "B",
            "is_active": false
        });
        let _item2 = create_test_object(&client, &class_name, item2_data.clone()).await?;

        // Test with select("name", "category")
        let mut query_select = ParseQuery::new(&class_name);
        query_select.select(&["name", "category"]); // Select only name and category

        // We'll deserialize into GenericItem to check field presence dynamically
        let results_select: Vec<GenericItem> = query_select
            .find(&client)
            .await
            .expect("Query with select failed");

        assert_eq!(results_select.len(), 2, "Expected two items with select");

        for item in results_select {
            assert!(
                item.fields.contains_key("name"),
                "Selected field 'name' should be present"
            );
            assert!(
                item.fields.contains_key("category"),
                "Selected field 'category' should be present"
            );
            assert!(
                !item.fields.contains_key("value"),
                "Unselected field 'value' should be absent"
            );
            assert!(
                !item.fields.contains_key("is_active"),
                "Unselected field 'is_active' should be absent"
            );

            // Standard fields like createdAt, updatedAt are usually returned by Parse even with select
            // objectId is part of GenericItem struct directly
            assert!(
                item.fields.contains_key("createdAt"),
                "'createdAt' should be present by default"
            );
            assert!(
                item.fields.contains_key("updatedAt"),
                "'updatedAt' should be present by default"
            );
        }

        // Test without select (all fields should be present)
        let query_no_select = ParseQuery::new(&class_name);
        let results_no_select: Vec<GenericItem> = query_no_select
            .find(&client)
            .await
            .expect("Query without select failed");

        assert_eq!(
            results_no_select.len(),
            2,
            "Expected two items without select"
        );
        for item in results_no_select {
            assert!(item.fields.contains_key("name"));
            assert!(item.fields.contains_key("value"));
            assert!(item.fields.contains_key("category"));
            assert!(item.fields.contains_key("is_active"));
        }

        cleanup_test_class(&client, &class_name).await;

        Ok(())
    }
}
