//!
//! # Parse SDK for Rust (parse-rs)
//!
//! A modern, asynchronous, and unofficial Rust SDK for interacting with [Parse Server](https://parseplatform.org/) backends.
//! This crate provides a type-safe and ergonomic way to perform operations such as user authentication,
//! object management (CRUD), querying, file handling, and executing Cloud Code functions.
//!
//! ## Overview
//!

//! `parse-rs` is built with `async/await` for non-blocking operations and leverages popular Rust libraries
//! like `reqwest` for HTTP communication and `serde` for JSON serialization/deserialization.
//!
//! For detailed setup, prerequisites, and advanced usage, please refer to the
//! [README.md](https://github.com/tbraun96/parse-rs/blob/main/README.md) file in the repository.
//!
//! ## Quick Start Example
//!
//! ```rust,no_run
//! use parse_rs::{Parse, ParseError, ParseObject, object::CreateObjectResponse};
//! use serde_json::Value;
//! use std::collections::HashMap;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), ParseError> {
//!     // Initialize the Parse client.
//!     // Ensure PARSE_SERVER_URL, PARSE_APP_ID, and PARSE_MASTER_KEY (or other keys)
//!     // are set in your environment or provide them directly.
//!     let server_url = std::env::var("PARSE_SERVER_URL").unwrap_or_else(|_| "http://localhost:1338/parse".to_string());
//!     let app_id = std::env::var("PARSE_APP_ID").unwrap_or_else(|_| "myAppId".to_string());
//!     let master_key = std::env::var("PARSE_MASTER_KEY").unwrap_or_else(|_| "myMasterKey".to_string());
//!
//!     let mut client = Parse::new(
//!         &server_url,
//!         &app_id,
//!         None, // javascript_key
//!         None, // rest_api_key
//!         Some(&master_key),
//!     )?;
//!
//!     // Create a new ParseObject
//!     let mut game_score_data = HashMap::new();
//!     game_score_data.insert("score".to_string(), Value::Number(1337.into()));
//!     game_score_data.insert("playerName".to_string(), Value::String("Sean Plott".to_string()));
//!
//!     let mut new_score = ParseObject::new("GameScore");
//!     let created_object: CreateObjectResponse = client.create_object("GameScore", &new_score).await?;
//!
//!     println!("Successfully created GameScore with objectId: {}", created_object.object_id);
//!     Ok(())
//! }
//! ```
//!
//! For more examples and detailed API documentation, please explore the individual modules.

pub mod acl;
pub mod analytics;
pub mod client;
pub mod cloud;
pub mod config;
pub mod error;
pub mod file;
pub mod geopoint;
pub mod installation;
pub mod object;
pub mod query;
pub mod relations;
pub mod requests;
pub mod role;
/// Module for defining Parse Server class schemas and their fields.
pub mod schema;
pub mod session;
pub mod types;
pub mod user;

/// Represents a Parse Access Control List. See [`acl::ParseACL`](acl/struct.ParseACL.html) for details.
pub use acl::ParseACL;
/// The main client for interacting with a Parse Server.
/// See [`client::Parse`](client/struct.Parse.html) for detailed API methods and usage examples.
pub use client::Parse;
/// Handler for Parse Cloud Code functions. See [`cloud::ParseCloud`](cloud/struct.ParseCloud.html) for details on how to call functions.
pub use cloud::ParseCloud;
/// Represents server configuration retrievable via the Parse API. See [`config::ParseConfig`](config/struct.ParseConfig.html).
pub use config::ParseConfig;
/// Represents errors that can occur when interacting with Parse Server.
/// See [`error::ParseError`](error/enum.ParseError.html) for different error variants and their meanings.
pub use error::ParseError;
/// Represents a file stored in Parse Server. See [`file::ParseFile`](file/struct.ParseFile.html) for details on uploading and managing files.
pub use file::{FileField, ParseFile};
/// Represents a generic Parse Object, the fundamental data unit in Parse.
/// See [`object::ParseObject`](object/struct.ParseObject.html) for details on creating, retrieving, updating, and deleting objects.
pub use object::{ParseObject, RetrievedParseObject};
/// Used to construct and execute queries against Parse Server.
/// See [`query::ParseQuery`](query/struct.ParseQuery.html) for building complex queries with various constraints.
pub use query::ParseQuery;
/// Represents a Parse Role, used for managing groups of users and their permissions.
/// See [`role::ParseRole`](role/struct.ParseRole.html) for details.
pub use role::{NewParseRole, ParseRole};
/// Structs and enums related to Parse Server class schemas.
/// See the [`schema`](schema/index.html) module for more information.
pub use schema::{
    ClassLevelPermissionsSchema, FieldSchema, FieldType, GetAllSchemasResponse, ParseSchema,
};
/// Represents a Parse Session, linking a user to their logged-in state.
/// See [`session::ParseSession`](session/struct.ParseSession.html) for details.
pub use session::ParseSession;
/// Contains common Parse-specific data types like `ParseDate` and `Pointer`.
/// See the [`types`](types/index.html) module for more information.
pub use types::{
    Endpoint, ParseDate, ParseRelation, Pointer, QueryParams, RelationOp, Results,
    UpdateResponseData,
};
/// Represents a Parse User, handling authentication and user-specific data.
/// See [`user::ParseUser`](user/struct.ParseUser.html) for details.
pub use user::{LoginRequest, ParseUser, PasswordResetRequest, SignupRequest, SignupResponse};
