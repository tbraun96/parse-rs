// src/query.rs

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::{client::Parse, error::ParseError, Pointer};

/// Represents a query to be performed against a Parse Server class.
#[derive(Debug, Clone)]
pub struct ParseQuery {
    class_name: String,
    conditions: Map<String, Value>,
    limit: Option<isize>, // Parse uses signed int for limit, -1 for no limit (though we might handle Some(0) as no limit client side)
    skip: Option<usize>,
    order: Option<String>,
    include: Option<String>,
    keys: Option<String>, // For selecting specific fields
    // count_flag: bool, // To indicate if this is a count query, managed by the count() method call
    // read_preference: Option<String>, // For advanced MongoDB read preferences, future
    // include_all: bool, // Future
    use_master_key: bool, // Whether the query should be executed with the master key
}

impl ParseQuery {
    /// Creates a new `ParseQuery` for the specified class name.
    ///
    /// # Arguments
    /// * `class_name` - The name of the Parse class to query.
    pub fn new(class_name: &str) -> Self {
        Self {
            class_name: class_name.to_string(),
            conditions: Map::new(),
            limit: None,
            skip: None,
            order: None,
            include: None,
            keys: None,
            // count_flag: false,
            use_master_key: false, // Default to false
        }
    }

    /// Returns the class name this query targets.
    pub fn class_name(&self) -> &str {
        &self.class_name
    }

    /// Checks if this query is configured to use the master key.
    pub fn uses_master_key(&self) -> bool {
        self.use_master_key
    }

    /// Sets whether this query should be executed using the master key.
    pub fn set_master_key(&mut self, use_key: bool) -> &mut Self {
        self.use_master_key = use_key;
        self
    }

    // Helper to add a simple condition like "field": "value"
    fn add_simple_condition(&mut self, key: &str, value: Value) -> &mut Self {
        self.conditions.insert(key.to_string(), value);
        self
    }

    // Helper to add an operator condition like "field": {"$op": "value"}
    fn add_operator_condition(&mut self, key: &str, operator: &str, value: Value) -> &mut Self {
        let mut op_map = Map::new();
        op_map.insert(operator.to_string(), value);
        self.conditions
            .insert(key.to_string(), Value::Object(op_map));
        self
    }

    /// Adds a constraint to the query that a field must be equal to a specified value.
    pub fn equal_to<V: Serialize>(&mut self, key: &str, value: V) -> &mut Self {
        match serde_json::to_value(value) {
            Ok(json_val) => self.add_simple_condition(key, json_val),
            Err(_) => {
                /* Handle error or log, for now, do nothing or panic */
                self
            }
        }
    }

    /// Adds a constraint to the query that a field must not be equal to a specified value.
    pub fn not_equal_to<V: Serialize>(&mut self, key: &str, value: V) -> &mut Self {
        match serde_json::to_value(value) {
            Ok(json_val) => self.add_operator_condition(key, "$ne", json_val),
            Err(_) => self,
        }
    }

    /// Adds a constraint to the query that a field must exist.
    pub fn exists(&mut self, key: &str) -> &mut Self {
        self.add_operator_condition(key, "$exists", serde_json::Value::Bool(true))
    }

    /// Adds a constraint to the query that a field must not exist.
    pub fn does_not_exist(&mut self, key: &str) -> &mut Self {
        self.add_operator_condition(key, "$exists", serde_json::Value::Bool(false))
    }

    /// Adds a constraint for finding objects where a field's value is greater than the provided value.
    pub fn greater_than<V: Serialize>(&mut self, key: &str, value: V) -> &mut Self {
        match serde_json::to_value(value) {
            Ok(json_val) => self.add_operator_condition(key, "$gt", json_val),
            Err(_) => self,
        }
    }

    /// Adds a constraint for finding objects where a field's value is greater than or equal to the provided value.
    pub fn greater_than_or_equal_to<V: Serialize>(&mut self, key: &str, value: V) -> &mut Self {
        match serde_json::to_value(value) {
            Ok(json_val) => self.add_operator_condition(key, "$gte", json_val),
            Err(_) => self,
        }
    }

    /// Adds a constraint for finding objects where a field's value is less than the provided value.
    pub fn less_than<V: Serialize>(&mut self, key: &str, value: V) -> &mut Self {
        match serde_json::to_value(value) {
            Ok(json_val) => self.add_operator_condition(key, "$lt", json_val),
            Err(_) => self,
        }
    }

    /// Adds a constraint for finding objects where a field's value is less than or equal to the provided value.
    pub fn less_than_or_equal_to<V: Serialize>(&mut self, key: &str, value: V) -> &mut Self {
        match serde_json::to_value(value) {
            Ok(json_val) => self.add_operator_condition(key, "$lte", json_val),
            Err(_) => self,
        }
    }

    /// Adds a constraint for finding objects where a field's value is contained in the provided list of values.
    pub fn contained_in<V: Serialize>(&mut self, key: &str, values: Vec<V>) -> &mut Self {
        match serde_json::to_value(values) {
            Ok(json_val_array) => self.add_operator_condition(key, "$in", json_val_array),
            Err(_) => self,
        }
    }

    /// Adds a constraint for finding objects where a field's value is not contained in the provided list of values.
    pub fn not_contained_in<V: Serialize>(&mut self, key: &str, values: Vec<V>) -> &mut Self {
        match serde_json::to_value(values) {
            Ok(json_val_array) => self.add_operator_condition(key, "$nin", json_val_array),
            Err(_) => self,
        }
    }

    /// Adds a constraint for finding objects where a field contains all of the provided values (for array fields).
    pub fn contains_all<V: Serialize>(&mut self, key: &str, values: Vec<V>) -> &mut Self {
        match serde_json::to_value(values) {
            Ok(json_val_array) => self.add_operator_condition(key, "$all", json_val_array),
            Err(_) => self,
        }
    }

    /// Adds a constraint for finding objects where a string field starts with a given prefix.
    pub fn starts_with(&mut self, key: &str, prefix: &str) -> &mut Self {
        self.add_operator_condition(
            key,
            "$regex",
            Value::String(format!("^{}", regex::escape(prefix))),
        )
    }

    /// Adds a constraint for finding objects where a string field ends with a given suffix.
    pub fn ends_with(&mut self, key: &str, suffix: &str) -> &mut Self {
        self.add_operator_condition(
            key,
            "$regex",
            Value::String(format!("{}$", regex::escape(suffix))),
        )
    }

    /// Adds a constraint for finding objects where a string field contains a given substring.
    /// This uses a regex `.*substring.*`.
    pub fn contains(&mut self, key: &str, substring: &str) -> &mut Self {
        self.add_operator_condition(
            key,
            "$regex",
            Value::String(format!(".*{}.*", regex::escape(substring))),
        )
    }

    /// Adds a constraint for finding objects where a string field matches a given regex pattern.
    /// Modifiers can be 'i' for case-insensitive, 'm' for multiline, etc.
    pub fn matches_regex(
        &mut self,
        key: &str,
        regex_pattern: &str,
        modifiers: Option<&str>,
    ) -> &mut Self {
        let mut regex_map = Map::new();
        regex_map.insert(
            "$regex".to_string(),
            Value::String(regex_pattern.to_string()),
        );
        if let Some(mods) = modifiers {
            regex_map.insert("$options".to_string(), Value::String(mods.to_string()));
        }
        self.conditions
            .insert(key.to_string(), Value::Object(regex_map));
        self
    }

    /// Adds a constraint for full-text search on a field.
    /// Requires a text index to be configured on the field in MongoDB.
    ///
    /// # Arguments
    /// * `key` - The field to perform the text search on.
    /// * `term` - The search term.
    /// * `language` - Optional: The language for the search (e.g., "en", "es").
    /// * `case_sensitive` - Optional: Whether the search should be case-sensitive.
    /// * `diacritic_sensitive` - Optional: Whether the search should be diacritic-sensitive.
    pub fn search(
        &mut self,
        key: &str,
        term: &str,
        language: Option<&str>,
        case_sensitive: Option<bool>,
        diacritic_sensitive: Option<bool>,
    ) -> &mut Self {
        let mut search_params_map = Map::new();
        search_params_map.insert("$term".to_string(), Value::String(term.to_string()));

        if let Some(lang) = language {
            search_params_map.insert("$language".to_string(), Value::String(lang.to_string()));
        }
        if let Some(cs) = case_sensitive {
            search_params_map.insert("$caseSensitive".to_string(), Value::Bool(cs));
        }
        if let Some(ds) = diacritic_sensitive {
            search_params_map.insert("$diacriticSensitive".to_string(), Value::Bool(ds));
        }

        let mut search_op = Map::new();
        search_op.insert("$search".to_string(), Value::Object(search_params_map));

        let mut text_op = Map::new();
        text_op.insert("$text".to_string(), Value::Object(search_op));

        self.conditions
            .insert(key.to_string(), Value::Object(text_op));
        self
    }

    /// Adds a constraint to the query that objects must be related to a given parent object
    /// through a specific relation field.
    ///
    /// # Arguments
    /// * `parent_object` - A `Pointer` to the parent object.
    /// * `key_on_parent_object` - The name of the relation field on the `parent_object`.
    ///
    /// Example: Querying for "Comment" objects related to a "Post" object via the "comments" relation field on "Post":
    /// ```
    /// // let post_pointer = Pointer::new("Post", "postId123");
    /// // let mut comment_query = ParseQuery::new("Comment");
    /// // comment_query.related_to(&post_pointer, "comments");
    /// ```
    /// This will find all "Comment" objects that are part of the "comments" relation of the specified "Post".
    pub fn related_to(&mut self, parent_object: &Pointer, key_on_parent_object: &str) -> &mut Self {
        let mut related_to_map = Map::new();
        match serde_json::to_value(parent_object) {
            Ok(parent_ptr_json) => {
                related_to_map.insert("object".to_string(), parent_ptr_json);
                related_to_map.insert(
                    "key".to_string(),
                    Value::String(key_on_parent_object.to_string()),
                );
                self.conditions
                    .insert("$relatedTo".to_string(), Value::Object(related_to_map));
            }
            Err(_) => {
                // Handle or log serialization error for parent_object
                // For now, effectively a no-op if serialization fails, which is not ideal.
                // Consider returning Result<&mut Self, Error> or panicking for critical errors.
            }
        }
        self
    }

    // --- Pagination and Sorting ---

    /// Sets the maximum number of results to return.
    pub fn limit(&mut self, count: isize) -> &mut Self {
        self.limit = Some(count);
        self
    }

    /// Sets the number of results to skip before returning.
    pub fn skip(&mut self, count: usize) -> &mut Self {
        self.skip = Some(count);
        self
    }

    /// Sets the order of the results. Replaces any existing order.
    /// Takes a comma-separated string of field names. Prefix with '-' for descending order.
    /// e.g., "score,-playerName"
    pub fn order(&mut self, field_names: &str) -> &mut Self {
        self.order = Some(field_names.to_string());
        self
    }

    // Helper to append to the order string
    fn append_order_key(&mut self, key: &str, descending: bool) {
        let prefix = if descending { "-" } else { "" };
        let new_order_key = format!("{}{}", prefix, key);
        if let Some(existing_order) = &mut self.order {
            if !existing_order.is_empty() {
                existing_order.push(',');
            }
            existing_order.push_str(&new_order_key);
        } else {
            self.order = Some(new_order_key);
        }
    }

    /// Sorts the results by a given key in ascending order. Replaces existing sort order.
    pub fn order_by_ascending(&mut self, key: &str) -> &mut Self {
        self.order = Some(key.to_string());
        self
    }

    /// Sorts the results by a given key in descending order. Replaces existing sort order.
    pub fn order_by_descending(&mut self, key: &str) -> &mut Self {
        self.order = Some(format!("-{}", key));
        self
    }

    /// Adds a key to sort the results by in ascending order. Appends to existing sort order.
    pub fn add_ascending_order(&mut self, key: &str) -> &mut Self {
        self.append_order_key(key, false);
        self
    }

    /// Adds a key to sort the results by in descending order. Appends to existing sort order.
    pub fn add_descending_order(&mut self, key: &str) -> &mut Self {
        self.append_order_key(key, true);
        self
    }

    /// Includes nested ParseObjects for the given pointer key(s).
    /// The included field's data will be fetched and returned with the main object.
    pub fn include(&mut self, keys_to_include: &[&str]) -> &mut Self {
        let current_include = self.include.take().unwrap_or_default();
        let mut include_parts: Vec<&str> = current_include
            .split(',')
            .filter(|s| !s.is_empty())
            .collect();
        include_parts.extend(keys_to_include.iter().cloned());
        include_parts.sort_unstable(); // Optional: keep it sorted for consistency
        include_parts.dedup();
        self.include = Some(include_parts.join(","));
        self
    }

    /// Restricts the fields returned for all matching objects.
    pub fn select(&mut self, keys_to_select: &[&str]) -> &mut Self {
        let current_keys = self.keys.take().unwrap_or_default();
        let mut select_parts: Vec<&str> =
            current_keys.split(',').filter(|s| !s.is_empty()).collect();
        select_parts.extend(keys_to_select.iter().cloned());
        select_parts.sort_unstable(); // Optional: keep it sorted
        select_parts.dedup();
        self.keys = Some(select_parts.join(","));
        self
    }

    // --- Execution Methods ---

    // Internal helper to build query parameters for reqwest
    pub fn build_query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();
        if !self.conditions.is_empty() {
            if let Ok(where_json) = serde_json::to_string(&self.conditions) {
                params.push(("where".to_string(), where_json));
            }
        }
        if let Some(limit_val) = self.limit {
            params.push(("limit".to_string(), limit_val.to_string()));
        }
        if let Some(skip_val) = self.skip {
            params.push(("skip".to_string(), skip_val.to_string()));
        }
        if let Some(order_val) = &self.order {
            params.push(("order".to_string(), order_val.clone()));
        }
        if let Some(include_val) = &self.include {
            params.push(("include".to_string(), include_val.clone()));
        }
        if let Some(keys_val) = &self.keys {
            params.push(("keys".to_string(), keys_val.clone()));
        }
        params
    }

    async fn find_raw<T: DeserializeOwned + Send + Sync + 'static>(
        &self,
        client: &Parse,
    ) -> Result<FindResponse<T>, ParseError> {
        let endpoint = format!("classes/{}", self.class_name);
        let params = self.build_query_params();
        let response_wrapper: FindResponse<T> = client
            ._get_with_url_params(&endpoint, &params, self.use_master_key, None)
            .await?;
        Ok(response_wrapper)
    }

    async fn first_raw<T: DeserializeOwned + Send + Sync + 'static>(
        &self,
        client: &Parse,
    ) -> Result<Option<T>, ParseError> {
        let mut query_clone = self.clone();
        query_clone.limit(1);
        let endpoint = format!("classes/{}", query_clone.class_name);
        let params = query_clone.build_query_params();
        let response_wrapper: FindResponse<T> = client
            ._get_with_url_params(&endpoint, &params, self.use_master_key, None)
            .await?;
        Ok(response_wrapper.results.into_iter().next())
    }

    /// Retrieves a list of `ParseObject`s that match this query.
    pub async fn find<T: DeserializeOwned + Send + Sync + 'static>(
        &self,
        client: &Parse,
    ) -> Result<Vec<T>, ParseError> {
        let response_wrapper = self.find_raw(client).await?;
        Ok(response_wrapper.results)
    }

    /// Retrieves the first `ParseObject` that matches this query.
    pub async fn first<T: DeserializeOwned + Send + Sync + 'static>(
        &self,
        client: &Parse,
    ) -> Result<Option<T>, ParseError> {
        self.first_raw(client).await
    }

    /// Retrieves a specific `ParseObject` by its ID from the class associated with this query.
    /// Note: This method ignores other query constraints like `equalTo`, `limit`, etc., and directly fetches by ID.
    pub async fn get<T: DeserializeOwned + Send + Sync + 'static>(
        &self,
        object_id: &str,
        client: &Parse,
    ) -> Result<T, ParseError> {
        let endpoint = format!("classes/{}/{}", self.class_name, object_id);
        let params = self.build_query_params();
        client
            ._get_with_url_params(&endpoint, &params, self.use_master_key, None)
            .await
    }

    /// Counts the number of objects that match this query.
    pub async fn count(&self, client: &Parse) -> Result<u64, ParseError> {
        let mut query_clone = self.clone();
        query_clone.limit(0); // Limit 0 is for count

        let endpoint = format!("classes/{}", query_clone.class_name);
        let mut params = query_clone.build_query_params();
        params.push(("count".to_string(), "1".to_string()));

        let response_wrapper: CountResponse = client
            ._get_with_url_params(&endpoint, &params, self.use_master_key, None)
            .await?;
        Ok(response_wrapper.count)
    }

    /// Executes a distinct query for a specific field.
    /// Returns a vector of unique values for the given field that match the query conditions.
    pub async fn distinct<T: DeserializeOwned + Send + Sync + 'static>(
        &self,
        client: &Parse,
        field: &str,
    ) -> Result<Vec<T>, ParseError> {
        let endpoint = format!("aggregate/{}", self.class_name);

        let mut pipeline: Vec<Value> = Vec::new();

        // Add $match stage if there are 'where' conditions
        if !self.conditions.is_empty() {
            pipeline.push(json!({
                "$match": self.conditions
            }));
        }

        // Add $group stage for distinct operation
        pipeline.push(json!({
            "$group": { "_id": format!("${}", field) } // Use "_id" as the output field name
        }));

        // Serialize the pipeline
        let pipeline_json = serde_json::to_string(&pipeline).map_err(|e| {
            ParseError::SerializationError(format!(
                "Failed to serialize pipeline for distinct query: {}",
                e
            ))
        })?;

        let params = vec![("pipeline".to_string(), pipeline_json)];

        // The server returns { "results": [ { "objectId": value1 }, { "objectId": value2 }, ... ] }
        // despite the $group stage specifying { "_id": ... }. This is a Parse Server behavior.
        #[derive(serde::Deserialize, Debug)]
        struct DistinctItem<V> {
            #[serde(rename = "objectId")] // Parse Server returns the grouped key as "objectId"
            value: V,
        }

        let response_wrapper: FindResponse<DistinctItem<T>> = client
            ._get_with_url_params(&endpoint, &params, true, None) // Always use master key for aggregate
            .await?;

        // Extract the actual values from the DistinctItem wrappers
        let distinct_values = response_wrapper
            .results
            .into_iter()
            .map(|item| item.value)
            .collect();

        Ok(distinct_values)
    }

    /// Executes an aggregation query.
    ///
    /// The pipeline is a series of data aggregation steps. Refer to MongoDB aggregation pipeline documentation.
    /// Each stage in the pipeline should be a `serde_json::Value` object.
    /// This operation typically requires the master key.
    ///
    /// # Arguments
    /// * `pipeline` - A vector of `serde_json::Value` representing the aggregation stages.
    /// * `client` - The `Parse` to use for the request.
    ///
    /// # Returns
    /// A `Result` containing a `Vec<T>` of the deserialized results, or a `ParseError`.
    pub async fn aggregate<T: DeserializeOwned + Send + Sync + 'static>(
        &self,
        pipeline: Vec<Value>,
        client: &crate::client::Parse,
    ) -> Result<Vec<T>, crate::error::ParseError> {
        client
            .execute_aggregate(&self.class_name, serde_json::Value::Array(pipeline))
            .await
    }
}

#[derive(Debug, Deserialize)]
struct FindResponse<T> {
    results: Vec<T>,
}

#[derive(Debug, Deserialize)]
struct CountResponse {
    count: u64,
}

#[cfg(test)]
mod tests {
    // ... existing tests ...
}
