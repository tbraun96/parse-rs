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
* **Session Management:** Session tokens will be managed by the `Parse` client
  instance. Initially, this will be in-memory.

### 3.2. Object Management (`ParseObject`)

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
  * Constraint methods: `equalTo`, `notEqualTo`, `greaterThan`, `lessThan`,
    `containedIn`, `startsWith`, `exists`, `doesNotExist`, etc.
  * `find() -> Result<Vec<ParseObject>, ParseError>`: Retrieves all objects
    matching the query.
  * `first() -> Result<Option<ParseObject>, ParseError>`: Retrieves the first
    object matching the query.
  * `get(object_id: &str) -> Result<ParseObject, ParseError>`: Retrieves a
    specific object by its ID.
  * `count() -> Result<u64, ParseError>`: Counts objects matching the query.
  * Pagination: `limit(count: usize)`, `skip(count: usize)`.
  * Sorting: `orderByAscending(key: &str)`, `orderByDescending(key: &str)`.
  * Relational queries: `include(key: &str)` (for Pointers),
    `select(keys: Vec<&str>)`.

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
    └── acl.rs         # ParseACL struct and methods (Future)
    └── config.rs      # ParseConfig (Future)
    └── util.rs        # Common utility functions, if any
```

## 7. Implementation Checklist

### Phase 1: Core Setup & User Authentication

* [x] Setup project structure, `Cargo.toml` with dependencies (`tokio`,
  `reqwest`, `serde`, `serde_json`, `thiserror`, `url`).
* [x] Define `ParseError` enum (`src/error.rs`).
* [ ] Implement `Parse` client struct and `Parse::new()` (`src/lib.rs` or
  `src/client.rs`).
* [ ] Implement internal HTTP request sending logic (POST, GET, PUT, DELETE)
  with correct headers and error parsing (`src/client.rs`).
* [x] Define basic `ParseDate` type for `createdAt`, `updatedAt`
  (`src/types/date.rs`).
* [ ] Implement `ParseUser` struct (`src/user.rs`).
* [ ] Implement `ParseUser` methods: `signup`, `login`.
* [ ] Implement session token handling within the `Parse` client for
  authenticated requests.
* [ ] Implement `ParseUser` methods: `logout`, `become`, `current_user`
  (in-memory), `is_authenticated`, `get_session_token`.
* [ ] Basic integration tests for User Authentication against a live Parse
  Server.

### Phase 2: Object Management

* [ ] Define `ParseObject` struct (`src/object.rs`), initially using
  `serde_json::Value` for custom fields.
* [ ] Implement `ParseObject::new(class_name: &str)`.
* [ ] Implement `ParseObject::set()` and `ParseObject::get()` for basic data
  types.
* [ ] Implement `ParseObject::save()` (handling both create and update).
* [ ] Implement `ParseObject::fetch()`.
* [ ] Implement `ParseObject::delete()`.
* [ ] Handle `objectId`, `createdAt`, `updatedAt` fields automatically.
* [ ] Basic integration tests for Object CRUD operations.

### Phase 3: Querying

* [ ] Define `ParseQuery` struct (`src/query.rs`).
* [ ] Implement `ParseQuery::new(class_name: &str)`.
* [ ] Implement basic constraint methods: `equalTo`, `notEqualTo`,
  `greaterThan`, `lessThan`.
* [ ] Implement `ParseQuery::find()`.
* [ ] Implement `ParseQuery::first()`.
* [ ] Implement `ParseQuery::get(object_id: &str)`.
* [ ] Implement `limit()`, `skip()`, `orderByAscending()`,
  `orderByDescending()`.
* [ ] Basic integration tests for Querying.

### Phase 4: Cloud Code & Advanced Features

* [ ] Implement `parse.cloud().run()` functionality (`src/cloud.rs`).
* [ ] Integration tests for Cloud Code.
* [ ] Implement `ParseObject` field operations: `increment`, `decrement`,
  `add_to_array`, `add_unique_to_array`, `remove_from_array`.
* [ ] Implement `ParseQuery` advanced constraints: `containedIn`,
  `notContainedIn`, `exists`, `doesNotExist`, `startsWith`, `endsWith`,
  `matchesRegex`.
* [ ] Implement `ParseQuery` relational queries: `include(key: &str)`,
  `select(keys: Vec<&str>)`.
* [ ] Define and integrate `ParsePointer` type (`src/types/pointer.rs`).

### Phase 5: Refinements, Files, and Documentation

* [ ] Implement `ParseFile` functionality (`src/file.rs`): creating from data,
  saving (uploading), getting URL.
* [ ] Integration tests for `ParseFile`.
* [ ] Add comprehensive public API documentation (doc comments).
* [ ] Create examples for common use cases in `README.md` or an `examples/`
  directory.
* [ ] Refine error handling, type safety, and overall API ergonomics.
* [ ] Consider advanced types/features:
  * [ ] `ParseGeoPoint` (`src/types/geopoint.rs`) and geo queries.
  * [ ] `ParseRelation`.
  * [ ] `ParseACL` (`src/acl.rs`).
  * [ ] `ParseRole`.
  * [ ] `ParseConfig`.
* [ ] Publish initial version to `crates.io`.

## 8. Development Environment & Testing

* A running Parse Server instance is required for integration testing.
  * Example Docker setup: `docker run -d --name parse-server-ephemeral -p ${PARSE_SERVER_PORT}:${PARSE_SERVER_PORT} -e
    APP_ID=myAppId -e MASTER_KEY=myMasterKey -e JAVASCRIPT_KEY=myJavascriptKey
    -e REST_API_KEY=myRestApiKey parseplatform/parse-server`
* Tests should cover all public API methods and various scenarios.
