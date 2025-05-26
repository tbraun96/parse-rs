use crate::error::ParseError;
use crate::types::RelationOp; // For types not directly re-exported at crate root like RelationOp
use crate::{ParseDate, Pointer};

use reqwest::Method;
use serde::Deserialize;

impl crate::Parse {
    /// Adds target objects to a relation field of a parent object.
    ///
    /// # Arguments
    /// * `parent_class_name`: The class name of the parent object.
    /// * `parent_object_id`: The object ID of the parent object.
    /// * `relation_key`: The key (field name) of the relation on the parent object.
    /// * `targets`: A slice of `Pointer`s representing the objects to add to the relation.
    ///
    /// # Returns
    /// A `Result` containing the `ParseDate` of the update or a `ParseError`.
    /// This operation typically requires the Master Key or appropriate ACLs.
    pub async fn add_to_relation(
        &self,
        parent_class_name: &str,
        parent_object_id: &str,
        relation_key: &str,
        targets: &[Pointer],
    ) -> Result<ParseDate, ParseError> {
        if targets.is_empty() {
            return Err(ParseError::InvalidInput(
                "targets cannot be empty for AddRelation operation.".to_string(),
            ));
        }
        if parent_class_name.is_empty() || parent_object_id.is_empty() || relation_key.is_empty() {
            return Err(ParseError::InvalidInput(
                "parent_class_name, parent_object_id, and relation_key cannot be empty."
                    .to_string(),
            ));
        }

        let endpoint = format!("classes/{}/{}", parent_class_name, parent_object_id);
        let relation_op = RelationOp::add(targets);
        let body = serde_json::json!({ relation_key: relation_op });

        let use_master_key = self.master_key.is_some();
        let session_token_to_use = if use_master_key {
            None
        } else {
            self.session_token.as_deref()
        };

        #[derive(Deserialize)]
        struct UpdateResponse {
            #[serde(rename = "updatedAt")]
            updated_at: String,
        }

        let response: UpdateResponse = self
            ._request(
                Method::PUT,
                &endpoint,
                Some(&body),
                use_master_key,
                session_token_to_use,
            )
            .await?;

        Ok(ParseDate::new(response.updated_at))
    }

    /// Removes target objects from a relation field of a parent object.
    ///
    /// # Arguments
    /// * `parent_class_name`: The class name of the parent object.
    /// * `parent_object_id`: The object ID of the parent object.
    /// * `relation_key`: The key (field name) of the relation on the parent object.
    /// * `targets`: A slice of `Pointer`s representing the objects to remove from the relation.
    ///
    /// # Returns
    /// A `Result` containing the `ParseDate` of the update or a `ParseError`.
    /// This operation typically requires the Master Key or appropriate ACLs.
    pub async fn remove_from_relation(
        &self,
        parent_class_name: &str,
        parent_object_id: &str,
        relation_key: &str,
        targets: &[Pointer],
    ) -> Result<ParseDate, ParseError> {
        if targets.is_empty() {
            return Err(ParseError::InvalidInput(
                "targets cannot be empty for RemoveRelation operation.".to_string(),
            ));
        }
        if parent_class_name.is_empty() || parent_object_id.is_empty() || relation_key.is_empty() {
            return Err(ParseError::InvalidInput(
                "parent_class_name, parent_object_id, and relation_key cannot be empty."
                    .to_string(),
            ));
        }

        let endpoint = format!("classes/{}/{}", parent_class_name, parent_object_id);
        let relation_op = RelationOp::remove(targets);
        let body = serde_json::json!({ relation_key: relation_op });

        let use_master_key = self.master_key.is_some();
        let session_token_to_use = if use_master_key {
            None
        } else {
            self.session_token.as_deref()
        };

        #[derive(Deserialize)]
        struct UpdateResponse {
            #[serde(rename = "updatedAt")]
            updated_at: String,
        }

        let response: UpdateResponse = self
            ._request(
                Method::PUT,
                &endpoint,
                Some(&body),
                use_master_key,
                session_token_to_use,
            )
            .await?;

        Ok(ParseDate::new(response.updated_at))
    }
}
