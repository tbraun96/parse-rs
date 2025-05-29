use crate::query_test_utils::shared::{
    cleanup_test_class, generate_unique_classname, setup_client,
};
use parse_rs::object::{CreateObjectResponse, RetrievedParseObject};
use parse_rs::ParseError;
use serde_json::json;

mod query_test_utils;

#[cfg(test)]
mod object_tests {
    use super::*;
    use parse_rs::Parse;

    async fn create_test_object_with_fields(
        client: &Parse,
        class_name: &str,
        some_field_value: &str,
        score_value: i32,
    ) -> CreateObjectResponse {
        let data = json!({
            "some_field": some_field_value,
            "score": score_value
        });
        client
            .create_object(class_name, &data)
            .await
            .expect("Failed to create test object with fields")
    }

    #[tokio::test]
    async fn test_retrieve_object_success() {
        let client = setup_client();
        let class_name = &generate_unique_classname("TestRetrieve");
        cleanup_test_class(&client, class_name).await;

        let initial_some_field = "hello_retrieve";
        let initial_score = 4242;
        let create_response =
            create_test_object_with_fields(&client, class_name, initial_some_field, initial_score)
                .await;
        let object_id = create_response.object_id;

        let retrieve_result = client.retrieve_object(class_name, &object_id).await;
        assert!(
            retrieve_result.is_ok(),
            "Failed to retrieve object: {:?}",
            retrieve_result.err()
        );
        let retrieved_object: RetrievedParseObject = retrieve_result.unwrap();

        assert_eq!(retrieved_object.object_id, object_id);
        assert_eq!(
            retrieved_object
                .fields
                .get("some_field")
                .expect("some_field not found")
                .as_str()
                .expect("some_field not a string"),
            initial_some_field
        );
        assert_eq!(
            retrieved_object
                .fields
                .get("score")
                .expect("score not found")
                .as_i64()
                .expect("score not an i64"),
            initial_score as i64
        );
        assert!(!retrieved_object.created_at.iso.is_empty());
        assert!(!retrieved_object.updated_at.iso.is_empty());
        assert!(retrieved_object.acl.is_none());

        cleanup_test_class(&client, class_name).await;
    }

    #[tokio::test]
    async fn test_update_object_success() {
        let client = setup_client();
        let class_name = &generate_unique_classname("TestUpdate");
        cleanup_test_class(&client, class_name).await;

        let initial_some_field = "initial_value";
        let initial_score = 100;
        let create_response =
            create_test_object_with_fields(&client, class_name, initial_some_field, initial_score)
                .await;
        let object_id = create_response.object_id;

        let retrieved_before_update: RetrievedParseObject = client
            .retrieve_object(class_name, &object_id)
            .await
            .expect("Failed to retrieve object before update");
        let initial_updated_at = retrieved_before_update.updated_at.clone();

        assert_eq!(
            retrieved_before_update
                .fields
                .get("some_field")
                .unwrap()
                .as_str()
                .unwrap(),
            initial_some_field
        );
        assert_eq!(
            retrieved_before_update
                .fields
                .get("score")
                .unwrap()
                .as_i64()
                .unwrap(),
            initial_score as i64
        );

        let updated_some_field = "updated_value";
        let update_data = json!({
            "some_field": updated_some_field
        });
        let update_response = client
            .update_object(class_name, &object_id, &update_data)
            .await;
        assert!(
            update_response.is_ok(),
            "Failed to update object: {:?}",
            update_response.err()
        );

        let retrieved_after_update: RetrievedParseObject = client
            .retrieve_object(class_name, &object_id)
            .await
            .expect("Failed to retrieve object after update");

        assert_eq!(retrieved_after_update.object_id, object_id);
        assert_eq!(
            retrieved_after_update
                .fields
                .get("some_field")
                .unwrap()
                .as_str()
                .unwrap(),
            updated_some_field
        );
        assert_eq!(
            retrieved_after_update
                .fields
                .get("score")
                .unwrap()
                .as_i64()
                .unwrap(),
            initial_score as i64,
            "Score should not have changed"
        );
        assert_ne!(
            retrieved_after_update.updated_at, initial_updated_at,
            "updatedAt should change after update"
        );
        assert_eq!(
            retrieved_after_update.created_at,
            retrieved_before_update.created_at
        );

        cleanup_test_class(&client, class_name).await;
    }

    #[tokio::test]
    async fn test_retrieve_object_not_found() {
        let client = setup_client();
        let class_name = "NonExistentClassForRetrieve";
        let non_existent_object_id = "123abcDEFxyz";

        let retrieve_result: Result<RetrievedParseObject, _> = client
            .retrieve_object(class_name, non_existent_object_id)
            .await;
        assert!(retrieve_result.is_err());

        match retrieve_result.err().unwrap() {
            ParseError::ObjectNotFound(message) => {
                assert!(message.contains("(101)"), "Error message was: {}", message);
            }
            e => panic!("Expected ObjectNotFound, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_retrieve_object_invalid_input() {
        let client = setup_client();
        let valid_class_name = "ValidClassForInputTest";
        let valid_object_id = "someValidId";

        let res_empty_class = client.retrieve_object("", valid_object_id).await;
        assert!(matches!(res_empty_class, Err(ParseError::InvalidInput(_))));

        let res_empty_id = client.retrieve_object(valid_class_name, "").await;
        assert!(matches!(res_empty_id, Err(ParseError::InvalidInput(_))));

        let res_invalid_class_start = client.retrieve_object("1Class", valid_object_id).await;
        assert!(matches!(
            res_invalid_class_start,
            Err(ParseError::InvalidInput(msg)) if msg.contains("must start with a letter or underscore")
        ));

        let res_invalid_class_char = client.retrieve_object("Class-Name", valid_object_id).await;
        assert!(matches!(
            res_invalid_class_char,
            Err(ParseError::InvalidInput(msg)) if msg.contains("can only contain letters, numbers, or underscores")
        ));
    }

    #[tokio::test]
    async fn test_delete_object_success() {
        let client = setup_client();
        let class_name = &generate_unique_classname("TestDelete");
        cleanup_test_class(&client, class_name).await;

        let create_response =
            create_test_object_with_fields(&client, class_name, "to_be_deleted", 555).await;
        let object_id = create_response.object_id;

        let delete_response = client.delete_object(class_name, &object_id).await;
        assert!(
            delete_response.is_ok(),
            "Failed to delete object: {:?}",
            delete_response.err()
        );

        let retrieve_result: Result<RetrievedParseObject, _> =
            client.retrieve_object(class_name, &object_id).await;
        assert!(retrieve_result.is_err());

        match retrieve_result.err().unwrap() {
            ParseError::ObjectNotFound(message) => {
                assert!(message.contains("(101)"), "Error message was: {}", message);
            }
            e => panic!("Expected ObjectNotFound after delete, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_delete_object_not_found() {
        let client = setup_client();
        let class_name = "NonExistentClassForDelete";
        let non_existent_object_id = "zyx987CBAfed";

        let delete_response = client
            .delete_object(class_name, non_existent_object_id)
            .await;
        assert!(delete_response.is_err());

        match delete_response.err().unwrap() {
            ParseError::ObjectNotFound(message) => {
                assert!(message.contains("(101)"), "Error message was: {}", message);
            }
            e => panic!(
                "Expected ObjectNotFound for deleting non-existent object, got {:?}",
                e
            ),
        }
    }

    #[tokio::test]
    async fn test_delete_object_invalid_input() {
        let client = setup_client();
        let valid_class_name = "ValidClassForDeleteInputTest";
        let valid_object_id = "someValidIdForDelete";

        let res_empty_class = client.delete_object("", valid_object_id).await;
        assert!(matches!(res_empty_class, Err(ParseError::InvalidInput(_))));

        let res_empty_id = client.delete_object(valid_class_name, "").await;
        assert!(matches!(res_empty_id, Err(ParseError::InvalidInput(_))));

        let res_invalid_class_start = client.delete_object("1ClassDelete", valid_object_id).await;
        assert!(matches!(
            res_invalid_class_start,
            Err(ParseError::InvalidInput(msg)) if msg.contains("must start with a letter or underscore")
        ));

        let res_invalid_class_char = client.delete_object("Class-Delete", valid_object_id).await;
        assert!(matches!(
            res_invalid_class_char,
            Err(ParseError::InvalidInput(msg)) if msg.contains("can only contain letters, numbers, or underscores")
        ));
    }
}
