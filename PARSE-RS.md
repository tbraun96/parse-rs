# Parse SDK for Rust (parse-rs) - Implementation Plan

## 1. Introduction

The primary goal of this project is to develop a comprehensive, asynchronous
Parse Server client (SDK) written in pure Rust. This SDK will leverage
`async/await` powered by the `tokio` runtime.

The API design will aim to mirror the ergonomics and developer experience of the
official Parse TypeScript SDK where it makes sense for idiomatic Rust. A key
difference will be the avoidance of global/static state; instead, a `Parse`
client instance will be explicitly created and used.

## 2. Core Client (`Parse` struct)

The main entry point for interacting with the Parse Server.

### Instantiation

```rust
// Example
let parse_client = Parse::new(
    "http://localhost:1338/parse", // Server URL
    "myAppId",                     // Application ID
    Some("myJavascriptKey"),       // Optional JavaScript Key
    None,                          // Optional REST API Key
    Some("myMasterKey")            // Optional Master Key
).await?; 
// Or a non-async new if initialization is cheap, with async methods for
// operations
```

* **Parameters:**
  * `server_url: String`: The full URL to the Parse Server API (e.g.,
    `https://api.parse.com/1` or `http://localhost:1338/parse`).
  * `app_id: String`: The Application ID for your Parse app.
  * `javascript_key: Option<String>`: The JavaScript Key (optional).
  * `rest_api_key: Option<String>`: The REST API Key (optional).
  * `master_key: Option<String>`: The Master Key (optional, grants
    unrestricted access).
* Internally, it will initialize an HTTP client (e.g., `reqwest`) configured
  with these credentials.

### Error Handling

A custom `ParseError` enum will be defined to represent various errors that can
occur, including network issues, Parse API errors (with error codes and
messages), and serialization/deserialization problems.

## 3. Key Modules & Functionality (Initial Priorities)

### 3.1. User Authentication (`parse.user()`)

This module will handle user sign-up, login, session management, and other
user-related operations.
It will likely return a `ParseUser` struct or a similar handler.

* **`ParseUser` struct:** Represents a Parse User object.
* **`ParseSession` struct:** Represents a Parse Session object, containing session details like `sessionToken`, `user` (pointer), `createdWith`, `expiresAt`, `installationId`.
* **Methods (on a handler returned by `parse.user()` or `ParseUser` static
  methods associated with a client instance):**
  * `signup(username: &str, password: &str, email: Option<&str>,
    other_fields: Option<serde_json::Value>) -> Result<ParseUser, ParseError>`
  * `login(username: &str, password: &str) -> Result<ParseUser, ParseError>`
  * `logout() -> Result<(), ParseError>`: Invalidates the current user's
    session token on the server and clears it locally.
  * `become(session_token: &str) -> Result<ParseUser, ParseError>`:
    Authenticates as a user with a given session token.
  * `current_user() -> Option<ParseUser>`: Retrieves the currently
    authenticated user (initially in-memory, persistent storage can be a future
    enhancement).
  * `request_password_reset(email: &str) -> Result<(), ParseError>`
  * `is_authenticated() -> bool`
  * `get_session_token() -> Option<String>`
* **Session Management (on a handler returned by `parse.session()`):**
  * `get_current_session() -> Result<ParseSession, ParseError>`: Retrieves the current session details for the authenticated user.
  * `get_all_sessions(query_params: Option<&str>) -> Result<Vec<ParseSession>, ParseError>`: Retrieves all sessions, optionally filtered by query parameters. Requires master key.
  * `get_session_by_object_id(object_id: &str) -> Result<ParseSession, ParseError>`: Retrieves a specific session by its object ID. Requires master key.
  * `update_session_by_object_id(object_id: &str, data: &serde_json::Value) -> Result<ParseDate, ParseError>`: Updates a specific session. Requires master key.
  * `delete_session_by_object_id(object_id: &str) -> Result<(), ParseError>`: Deletes a specific session. Requires master key.
* **Session Token Management:** Session tokens will be managed by the `Parse` client
  instance. Initially, this will be in-memory.

### 3.7. Roles (`ParseRole`)

For creating, retrieving, updating, and deleting Parse objects (rows in your
classes/tables).

* **`ParseObject` struct:** A somewhat generic struct to represent a Parse
  object. It could use `serde_json::Value` for fields or allow for strongly-typed
  objects via generics or macros.
  * `object_id: Option<String>`
  * `created_at: Option<ParseDate>`
  * `updated_at: Option<ParseDate>`
  * `acl: Option<ParseAcl>` (Future)
  * `fields: std::collections::HashMap<String, serde_json::Value>` (or similar)
* **Methods:**
  * `ParseObject::new(class_name: &str) -> Self`
  * `save(&mut self) -> Result<(), ParseError>`: Creates the object if
    `object_id` is `None`, otherwise updates it.
  * `fetch(&mut self) -> Result<(), ParseError>`: Fetches the latest data for
    the object from the server. Requires `object_id`.
  * `delete(self) -> Result<(), ParseError>`: Deletes the object from the
    server. Requires `object_id`.
  * `get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<T,
    ParseError>`
  * `set<T: serde::Serialize>(&mut self, key: &str, value: T) -> Result<(),
    ParseError>`
  * Field operations: `increment`, `add_to_array`, `remove_from_array`, etc.
* **Data Types:** Support for standard Parse data types (String, Number,
  Boolean, Array, Object, Date, Pointer, Relation, File, GeoPoint). `serde` will
  be crucial here.

### 3.3. Querying (`ParseQuery`)

For finding and retrieving `ParseObject`s based on various criteria.

* **`ParseQuery` struct:**
  * `ParseQuery::new(class_name: &str) -> Self`
* **Methods:**
  * Constraint methods:
    * `equalTo`, `notEqualTo`
    * `greaterThan`, `greaterThanOrEqualTo`
    * `lessThan`, `lessThanOrEqualTo`
  * `find() -> Result<Vec<ParseObject>, ParseError>`: Retrieves all objects
    matching the query.
  * `first() -> Result<Option<ParseObject>, ParseError>`: Retrieves the first
    object matching the query.
  * `get(object_id: &str) -> Result<ParseObject, ParseError>`: Retrieves a
    specific object by its ID.
  * `count() -> Result<u64, ParseError>`: Counts objects matching the query.
  * Pagination: `limit(count: usize)`, `skip(count: usize)`.
  * Sorting: `orderByAscending(key: &str)`, `orderByDescending(key: &str)`.
  * Relational queries:
    * `include(key: &str)`: Include the full data of a pointer field.
    * `include(keys: &[&str])`: Include the full data of multiple pointer fields.
    * `select(key: &str)`: Select only specific field(s) to be returned.
    * `select(keys: &[&str])`: Select only specific fields to be returned.

### 3.4. Cloud Code (`parse.cloud()`)

For calling Parse Cloud Code functions.

* **Methods (on a handler returned by `parse.cloud()`):**
  * `run<T: serde::de::DeserializeOwned>(function_name: &str,
    params: Option<serde_json::Value>) -> Result<T, ParseError>`

## 4. HTTP Layer

* **Client:** `reqwest` will be used for making asynchronous HTTP requests.
* **Headers:** The client will automatically manage necessary Parse headers:
  * `X-Parse-Application-Id`
  * `X-Parse-REST-API-Key` (if provided)
  * `X-Parse-JavaScript-Key` (if provided)
  * `X-Parse-Master-Key` (if provided)
  * `X-Parse-Session-Token` (when a user is authenticated)
  * `Content-Type: application/json` for request bodies.
* **Error Handling:** Parse API error responses (JSON format) will be parsed and
  converted into `ParseError`.

## 5. Serialization/Deserialization

* `serde` and `serde_json` will be used extensively for handling JSON request
  and response bodies, and for converting data to/from Rust structs.
* Custom `ParseDate`, `ParseGeoPoint`, `ParseFile`, `ParsePointer`,
  `ParseRelation` types might be needed for proper serialization and type safety.
* Successfully implemented custom deserialization logic for `ParseDate` using `serde`'s `deserialize_with`. This has been applied to `ParseObject`, `ParseUser`, `ParseRole`, and their associated response structs (e.g., `CreateObjectResponse`, `SignupResponse`, `CreateRoleResponse`), resolving previous test failures related to date string parsing from the Parse server.

## 6. Proposed Project Structure

```markdown
parse-rs/
├── Cargo.toml
├── PARSE-RS.md  # This document
└── src/
    ├── lib.rs         # Main module, Parse struct definition and new()
    ├── client.rs      # Core HTTP request logic, header management
    ├── error.rs       # ParseError enum and error handling utilities
    ├── types/         # Directory for Parse-specific data types
    │   ├── date.rs
    │   ├── geopoint.rs
    │   ├── pointer.rs
    │   └── mod.rs
    ├── user.rs        # ParseUser struct, authentication methods
    ├── object.rs      # ParseObject struct, CRUD operations
    ├── query.rs       # ParseQuery struct, querying methods
    ├── cloud.rs       # Cloud Code functions
    ├── file.rs        # ParseFile struct and methods (Stretch goal for initial phases)
    ├── acl.rs         # ParseACL struct and methods (Future)
    ├── config.rs      # ParseConfig (Future)
    ├── roles.rs       # ParseRole struct and methods (Future)
    ├── installation.rs # ParseInstallation struct and methods (Future)
    ├── push.rs        # Push Notification methods (Future)
    ├── schema.rs      # Schema management methods (Core CRUD and Fetching Implemented)
    └── util.rs        # Common utility functions, if any
```

## 7. Implementation Checklist

### Phase 1: Core Setup & User Authentication

## 8. Development Progress & Key Achievements

### May 30, 2025: Schema Management and Error Handling Enhancements

* **Schema Index Deserialization:**
  * Successfully implemented `IndexFieldType` enum (`SortOrder(i32)`, `Text(String)`, `Other(Value)`) within `src/schema.rs`.
  * This allows the SDK to correctly deserialize schema definitions that include non-numeric index types, such as "text" for full-text search, which was a previous blocker.
  * Updated `ParseSchema` to use `IndexFieldType` for its `indexes` field.

* **Integration Test Fixes (`schema_integration_tests.rs`):**
  * Resolved all failing tests in `tests/schema_integration_tests.rs`.
  * Corrected error handling logic in `test_create_get_and_delete_class_schema` and `test_delete_non_empty_class_schema_fails` to accurately expect `ParseError::OtherParseError` (with appropriate codes 103 and 255) instead of `ParseError::ApiError` for certain server responses. This involved understanding the error mapping in `ParseError::from_response`.
  * Updated debug logging for schema indexes to correctly use the new `IndexFieldType`.

* **Code Quality:**
  * Addressed compiler warnings in `tests/schema_integration_tests.rs` related to unused imports and mutable variables.

* **Overall Status:** The schema management functionality is now more robust, and related integration tests are passing, marking significant progress in ensuring the SDK can handle diverse Parse Server configurations and error responses correctly.

* [x] Setup project structure, `Cargo.toml` with dependencies (`tokio`,
  `reqwest`, `serde`, `serde_json`, `thiserror`, `url`).
* [x] Define `ParseError` enum (`src/error.rs`).
* [x] Implement `Parse` client struct and `Parse::new()` (`src/lib.rs` or
  `src/client.rs`).
* [x] Implement internal HTTP request sending logic (POST, GET, PUT, DELETE)
  with correct headers and error parsing (`src/client.rs`).
* [x] Define basic `ParseDate` type for `createdAt`, `updatedAt`
  (`src/types/date.rs`).
* [x] Implement `ParseUser` struct (`src/user.rs`).
* [x] Implement `ParseUser` methods: `signup`, `login`.
* [x] Implement `ParseUser` methods: `logout`, `become`, `current_user`, `request_password_reset`.
* [x] Implement basic session token handling in `Parse` client.
* [x] Define `ParseSession` struct and associated response types.
* [x] Implement `ParseSession` methods: `get_current_session` (me), `get_all_sessions`, `get_session_by_object_id`, `update_session_by_object_id`, `delete_session_by_object_id`.

### Phase 2: Object Management

* [x] Define `ParseObject` struct (`src/object.rs`), initially using
  `serde_json::Value` for custom fields.
* [x] Implement `ParseObject::new(class_name: &str)`.
* [x] Implement `ParseObject::set()` and `ParseObject::get()` for basic data
  types.
* [x] Implement `ParseObject::save()` (handling both create and update).
* [x] Implement `ParseObject::fetch()`.
* [x] Implement `ParseObject::delete()`.
* [x] Handle `objectId`, `createdAt`, `updatedAt` fields automatically.
* [x] Basic integration tests for Object CRUD operations.

### Phase 3: Querying

* [x] Define `ParseQuery` struct (`src/query.rs`).
* [x] Implement `ParseQuery::new(class_name: &str)`.
* [x] Implement basic constraint methods:
  * `equalTo`, `notEqualTo`
  * `greaterThan`, `greaterThanOrEqualTo`
  * `lessThan`, `lessThanOrEqualTo`
* [x] Implement `ParseQuery::find()`.
* [x] Implement `ParseQuery::first()`.
* [x] Implement `ParseQuery::get(object_id: &str)`.
* [x] Implement `ParseQuery::count()`.
* [x] Implement pagination: `limit(count: usize)`, `skip(count: usize)`.
* [x] Implement sorting: `orderByAscending(key: &str)`, `orderByDescending(key: &str)`.
* [x] Basic integration tests for Querying.
* [x] Implement `ParseQuery` relational queries:
  * `include(key: &str)`
  * `include(keys: &[&str])`
  * `select(key: &str)`
  * `select(keys: &[&str])`
* [x] Implement `ParseQuery` advanced constraints:
  * [x] `containedIn`
  * [x] `notContainedIn`
  * [x] `containsAll`
  * [x] `exists`, `doesNotExist`
  * [x] `startsWith`, `endsWith`, `contains`, `matchesRegex`
* [x] Implement `ParseQuery` aggregate queries:
  * [x] `aggregate`
* [x] Implement `ParseQuery` distinct values:
  * [x] `distinct`
* [ ] Implement `ParseQuery` geospatial queries (SKIP):
  * `nearSphere`, `withinGeoBox`
* [x] Implement `ParseQuery` full-text search:
  * [x] `search`

### Phase 4: Cloud Code & Advanced Features

* [x] Implement `parse.cloud().run()` functionality (`src/cloud.rs`).
* [x] Integration tests for Cloud Code.
* [x] Implement `ParseObject` field operations: `increment`, `decrement`,
  `add_to_array`, `add_unique_to_array`, `remove_from_array`.

### Phase 5: Refinements, Files, and Documentation

* [X] Implement `ParseFile` functionality (`src/file.rs`): creating from data,
  saving (uploading), getting URL.
* [X] Integration tests for `ParseFile`.
* [ ] Add comprehensive public API documentation (doc comments).
* [ ] Create examples for common use cases in `README.md` or an `examples/`
  directory.
* [ ] Consider advanced types/features:
  * [ ] `ParseInstallation` (for device registration, crucial for push notifications).
  * [ ] `ParsePush` (for sending push notifications).
  * [X] `ParseSchema` (for programmatically managing class schemas).

## 8. Development Updates

### 2025-05-24: Test Utilities Refactor & Bug Fixes

* **Test Utilities Refactoring**:
  * Encapsulated shared test functions and structs within `tests/query_test_utils.rs` into a public module named `shared`.
  * Updated all integration test files (`tests/query_*_integration.rs`) to use the new `shared` module (e.g., `use super::query_test_utils::shared::*;`), resolving previous scope and compilation errors.
* **Bug Fixes**:
  * **Aggregate Query Deserialization**: Fixed an issue where `test_aggregate_group_by_player_and_sum_scores` was failing due to a mismatch between the expected `_id` field and the actual `objectId` field returned by the Parse Server for grouped results. The `GroupedResult` struct in `tests/query_aggregate_ops_integration.rs` was updated accordingly. A memory was created regarding this server behavior.
  * **Count Query Parameter**: Resolved a bug in `ParseQuery::count()` where the `count=1` parameter was incorrectly being added to the `where` clause of the query. It's now correctly added as a top-level URL parameter, fixing deserialization failures (e.g., in `basic_ops_tests::test_query_initial_setup_and_find_empty`) where the server wouldn't return the `count` field.
  * **Code Cleanup**: Removed several unused import warnings across the test utilities and source files.
* **Outcome**: All integration tests are currently passing, and the test utility structure is more robust.

## 9. Development Environment & Testing

* A running Parse Server instance is required for integration testing.

* Example Docker setup:

```bash
docker run -d \
  --name parse-server-ephemeral-parse-rs \
  -p ${PARSE_SERVER_PORT}:${PARSE_SERVER_PORT} \
  -e APP_ID=myAppId \
  -e MASTER_KEY=myMasterKey \
  -e JAVASCRIPT_KEY=myJavascriptKey \
  -e REST_API_KEY=myRestApiKey \
  parseplatform/parse-server
```

* Tests should cover all public API methods and various scenarios.

## 10. Extended Phase - Future API Coverage

Beyond the immediate to-do list, the following Parse REST API features are candidates for future implementation to provide more comprehensive SDK coverage:

[x] **Sessions API**: (Finished)

* Direct management of user sessions (get current, query, delete specific sessions).
* Endpoints: `/parse/sessions`, `/parse/sessions/me`, `/parse/sessions/<objectId>`.

[ ] **User API Enhancements**:

* `[ ] Verifying User Emails`
* `[ ] Linking/Unlinking User Accounts (e.g., for social login, converting anonymous users)`

[x] **Roles API**:

* `[x] Full CRUD and management for Roles`
  * `[x] Creating Roles`
  * `[x] Retrieving Roles (by objectId, querying)`
  * `[x] Updating Roles (e.g., adding/removing users or other roles to a role's relations)`
  * `[x] Deleting Roles`

[x] **Analytics API**:

* `[x] Tracking custom application events (Endpoint: /parse/events/<eventName>)` (Basic `track_event` implemented)

[ ] **Installations API**:

* `[ ] Creating/Uploading Installation data`
* `[ ] Retrieving Installations (by objectId, querying)`
* `[ ] Updating Installations (e.g., channels, badge count, device token)`
* `[ ] Deleting Installations`

[ ] **Push Notifications API**:

* `[ ] Sending push notifications (to channels, segments, advanced targeting queries)`
* `[ ] Scheduling pushes`
* `[ ] Localizing pushes`

[ ] **Cloud Code Jobs (Advanced)**:

* While `ParseCloud` covers calling functions, more advanced job management (e.g., status polling, specific job retrieval if API supports, triggering background jobs) could be added.

[x] **Schemas API**:

* `[x] Programmatic schema management`
  * `[x] Fetching all schemas / specific class schema`
  * `[x] Creating a new class schema`
  * `[x] Modifying a class schema (adding/removing fields, constraints - if supported by API)`
  * `[x] Deleting a class schema`

[ ] **Hooks API**:

* Programmatic management of Cloud Function and Trigger webhooks.
* Endpoints: `/parse/hooks/functions`, `/parse/hooks/triggers`.

---
**Note**: This document is a living specification and will be updated as the project progresses.
