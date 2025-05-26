use parse_rs::{ParseObject, ParseQuery, Pointer};
use serde_json::json;

mod query_test_utils;
use query_test_utils::shared::{
    cleanup_test_class, generate_unique_classname, setup_client_with_master_key,
};

#[tokio::test]
async fn test_add_and_remove_relation() {
    let client = setup_client_with_master_key();
    let parent_class_name = generate_unique_classname("ParentRel");
    let child_class_name = generate_unique_classname("ChildRel");

    // 1. Create a parent object
    let mut parent_obj = ParseObject::new(&parent_class_name);
    parent_obj.set("name", json!("Test Parent"));
    let parent_obj_id = client
        .create_object(&parent_class_name, &parent_obj)
        .await
        .expect("Failed to create parent object")
        .object_id;

    let parent_pointer = Pointer::new(&parent_class_name, &parent_obj_id);

    // 2. Create a couple of child objects
    let mut child1 = ParseObject::new(&child_class_name);
    child1.set("name", json!("Child 1"));
    let child1_id = client
        .create_object(&child_class_name, &child1)
        .await
        .expect("Failed to create child1")
        .object_id;

    let mut child2 = ParseObject::new(&child_class_name);
    child2.set("name", json!("Child 2"));
    let child2_id = client
        .create_object(&child_class_name, &child2)
        .await
        .expect("Failed to create child2")
        .object_id;

    let child1_pointer = Pointer::new(&child_class_name, &child1_id);
    let child2_pointer = Pointer::new(&child_class_name, &child2_id);

    // 3. Add child1 and child2 to a relation "children" on parent_obj
    let relation_key = "children";
    client
        .add_to_relation(
            &parent_class_name,
            &parent_obj_id,
            relation_key,
            &[child1_pointer.clone(), child2_pointer.clone()],
        )
        .await
        .expect("Failed to add children to relation");

    // Verification after adding
    let mut query_after_add = ParseQuery::new(&child_class_name);
    query_after_add.related_to(&parent_pointer, relation_key);
    let related_children_after_add: Vec<ParseObject> = client
        .find_objects(&query_after_add)
        .await
        .expect("Failed to query related children after add");

    assert_eq!(
        related_children_after_add.len(),
        2,
        "Should find 2 related children after add"
    );
    let mut found_child1_after_add = false;
    let mut found_child2_after_add = false;
    for child in related_children_after_add.iter() {
        // Ensure child.object_id is Some before comparing
        if child.object_id == Some(child1_id.clone()) {
            found_child1_after_add = true;
        }
        if child.object_id == Some(child2_id.clone()) {
            found_child2_after_add = true;
        }
    }
    assert!(
        found_child1_after_add,
        "Child1 not found in relation after add"
    );
    assert!(
        found_child2_after_add,
        "Child2 not found in relation after add"
    );

    // 4. Remove child1 from the relation
    let remove_targets = vec![Pointer::new(&child_class_name, &child1_id)];
    let remove_result = client
        .remove_from_relation(
            &parent_class_name,
            &parent_obj_id,
            relation_key,
            &remove_targets,
        )
        .await;
    assert!(
        remove_result.is_ok(),
        "Failed to remove from relation: {:?}",
        remove_result.err()
    );

    // Verification after removing child1
    let mut query_after_remove = ParseQuery::new(&child_class_name);
    query_after_remove.related_to(&parent_pointer, relation_key);
    let related_children_after_remove: Vec<ParseObject> = client
        .find_objects(&query_after_remove)
        .await
        .expect("Failed to query related children after remove");

    assert_eq!(
        related_children_after_remove.len(),
        1,
        "Should only have one child after removing one"
    );
    assert_eq!(
        related_children_after_remove[0].object_id,
        Some(child2_id.clone()), // child1 was removed, child2 should remain
        "The remaining child should be child2"
    );

    // Cleanup
    let _ = client
        .delete_object(&parent_class_name, &parent_obj_id)
        .await;
    let _ = client.delete_object(&child_class_name, &child1_id).await;
    let _ = client.delete_object(&child_class_name, &child2_id).await;
    cleanup_test_class(&client, &parent_class_name).await;
    cleanup_test_class(&client, &child_class_name).await;
}
