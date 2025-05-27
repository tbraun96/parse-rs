# 🦀 Parse SDK for Rust (parse-rs)

[![Crates.io](https://img.shields.io/crates/v/parse-rs.svg?style=flat-square&logo=rust)](https://crates.io/crates/parse-rs)
[![Docs.rs](https://img.shields.io/docsrs/parse-rs?style=flat-square&logo=rust)](https://docs.rs/parse-rs)
[![Validate PR](https://github.com/tbraun96/parse-rs/actions/workflows/validate-pr.yml/badge.svg)](https://github.com/tbraun96/parse-rs/actions/workflows/validate-pr.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)

A modern asynchronous Rust SDK for interacting with [Parse Server](https://parseplatform.org/) backends. Effortlessly integrate your Rust applications with the powerful features of Parse.

---

## 📖 Overview

`parse-rs` aims to provide a comprehensive, easy-to-use, and type-safe interface for the Parse Server REST API. Whether you're building a new application or integrating with an existing Parse backend, this SDK is designed to streamline your development process.

This SDK is built with `async/await` for non-blocking operations and leverages popular Rust libraries like `reqwest` for HTTP communication and `serde` for JSON serialization/deserialization.

## 📋 Table of Contents

- [Features](#✨-features)
- [Prerequisites](#🛠️-prerequisites)
- [Installation](#🚀-installation)
- [Quick Start](#⚡-quick-start)
- [Running Tests](#🧪-running-tests)
- [API Coverage](#🎯-api-coverage)
- [Contributing](#🤝-contributing)
- [License](#📜-license)

## ✨ Features

- **Asynchronous API:** Built with `async/await` for efficient, non-blocking I/O.
- **User Authentication:** Sign up, log in, log out, session management.
- **Object Management:** Full CRUD (Create, Read, Update, Delete) operations for `ParseObject`s.
- **Powerful Queries:** Construct complex queries with constraints, sorting, pagination, relational queries, and aggregate functions.
- **Session Management:** Handle user sessions, including retrieval and revocation.
- **File Handling:** Upload and delete files associated with Parse objects.
- **Cloud Code:** Execute Cloud Code functions.
- **Configuration Management:** Retrieve server configuration.
- **Type Safety:** Leverages Rust's type system to minimize runtime errors.
- **Structured Error Handling:** Clear and descriptive error types.

*(For a detailed list of implemented and planned features, please see [PARSE-RS.md](./PARSE-RS.md#7-implementation-checklist))*.

## 🛠️ Prerequisites

- **Rust:** Version 1.65 or later
- **Parse Server:** A running instance of Parse Server. You can set one up locally using Docker or use a managed Parse hosting provider.
  - For local development and running integration tests, a `docker-compose.yml` is provided in this repository. See [Running Tests](#🧪-running-tests) for setup.
- **Parse Server Credentials:** You'll need your Application ID, and optionally your REST API Key, JavaScript Key, or Master Key depending on the operations you intend to perform.

## 🚀 Installation

Add `parse-rs` as a dependency to your `Cargo.toml` file:

```toml
[dependencies]
parse-rs = "0.1.0" # Replace with the latest version from crates.io
# Ensure you have tokio for the async runtime
tokio = { version = "1", features = ["full"] }
```

Then run `cargo build`.

*(Note: This SDK is not yet published to crates.io. This is a placeholder for when it is.)*

## ⚡ Quick Start

Here's a basic example of how to initialize the client and create a new object:

```rust
use parse_rs::{Parse, ParseObject, ParseError, Value};
use serde_json::json;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), ParseError> {
    // Initialize the Parse client
    // Ensure PARSE_SERVER_URL, PARSE_APP_ID, and PARSE_MASTER_KEY (or other keys) are set in your environment
    // or provide them directly.
    let server_url = std::env::var("PARSE_SERVER_URL").unwrap_or_else(|_| "http://localhost:1338/parse".to_string());
    let app_id = std::env::var("PARSE_APP_ID").unwrap_or_else(|_| "myAppId".to_string());
    let master_key = std::env::var("PARSE_MASTER_KEY").unwrap_or_else(|_| "myMasterKey".to_string());

    let mut client = Parse::new(
        &server_url,
        &app_id,
        None, // javascript_key
        None, // rest_api_key
        Some(&master_key),
    ).await?;

    // Create a new ParseObject
    let mut game_score_data = HashMap::new();
    game_score_data.insert("score".to_string(), Value::Number(1337.into()));
    game_score_data.insert("playerName".to_string(), Value::String("Sean Plott".to_string()));
    game_score_data.insert("cheatMode".to_string(), Value::Boolean(false));

    let mut new_score = ParseObject::new("GameScore", game_score_data);

    let created_object: ParseObject = client.create(&mut new_score).await?;

    println!("Successfully created GameScore with objectId: {}", created_object.get_object_id().unwrap());

    // Retrieve the object
    let retrieved_score: ParseObject = client.get("GameScore", created_object.get_object_id().unwrap()).await?;
    println!("Retrieved score for {}: {:?}", 
        retrieved_score.get_string("playerName").unwrap_or_default(),
        retrieved_score.get_number("score")
    );

    Ok(())
}
```

## 🧪 Running Tests

This SDK includes a suite of integration tests that run against a live Parse Server instance.

1. **Set up Parse Server:**
    A `docker-compose.yml` file is provided to easily spin up a Parse Server and MongoDB instance.

    ```bash
    # From the root of the repository
    docker compose up -d
    ```

    This will start Parse Server on `http://localhost:1338/parse`.

2. **Set Environment Variables:**
    The tests require certain environment variables to be set. You can create a `.env` file in the root of the project or set them in your shell:

    ```env
    PARSE_SERVER_URL=http://localhost:1338/parse
    PARSE_APP_ID=myAppId
    PARSE_MASTER_KEY=myMasterKey
    PARSE_JAVASCRIPT_KEY=myJavascriptKey
    PARSE_REST_API_KEY=myRestApiKey
    # Add any other keys if your server configuration requires them
    ```

    The `entrypoint.sh` script in the `parse-server-example` directory configures the server with these default credentials.

3. **Run Tests:**

    ```bash
    cargo test
    ```

4. **Tear Down Parse Server (Optional):**

    ```bash
    docker compose down
    ```

## 🎯 API Coverage

`parse-rs` aims to cover a significant portion of the Parse Server REST API.

**Currently Implemented:**

- User Authentication (Signup, Login, Logout, Get Current User, Session Token Validation)
- Object Management (Create, Retrieve, Update, Delete)
- Querying (Basic constraints, relational queries, pagination, ordering, aggregate)
- Session Management (Get Current Session, Get All Sessions, Revoke Session)
- File Upload & Deletion
- Cloud Code Function Execution
- Configuration Retrieval

For a detailed, up-to-date checklist of implemented features and future plans, please refer to the [PARSE-RS.md](./PARSE-RS.md#7-implementation-checklist) document.

## 🤝 Contributing

Contributions are welcome! Whether it's bug reports, feature requests, documentation improvements, or code contributions, please feel free to open an issue or submit a pull request.

Before contributing, please:

1. Read the [PARSE-RS.md](./PARSE-RS.md) document to understand the project's goals and current status.
2. Open an issue to discuss any significant changes or new features.
3. Ensure your code adheres to the existing style and passes all tests.
4. Add tests for any new functionality.

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.
