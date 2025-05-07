# AnyError

AnyError is a Rust wrapper for `dyn std::error::Error` types that enables serialization and maintains the error chain.

## Overview

When working with third-party libraries or building error enums with thiserror, AnyError lets you capture and serialize any error type while preserving their full context.

## Features

- **Error Wrapping**: Seamlessly wraps any `dyn Error` type
- **Serialization**: Full serde support for error serialization/deserialization
- **Error Chain Preservation**: Maintains nested error sources
- **Type Information**: Preserves standardized type names
- **Integration Support**: Works with thiserror, valuable, and serde

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
liberror = { version = "0.1.0", git = "https://github.com/charliethomson/liberror" }
```

## Usage with thiserror

AnyError works perfectly with thiserror to create domain-specific error types:

```rust
use liberror::AnyError;
use serde::{Serialize, Deserialize};
use thiserror::Error;

// Create a strongly-typed domain error enum
#[derive(Debug, Clone, Serialize, Deserialize, Error, valuable::Valuable)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "$type",
    content = "context"
)]
pub enum UserServiceError {
    #[error(transparent)]
    #[serde(rename = "app.service.user.database")]
    Database(AnyError),

    #[error(transparent)]
    #[serde(rename = "app.service.user.auth")]
    Authentication(AnyError),

    #[error("User not found: {0}")]
    #[serde(rename = "app.service.user.not_found")]
    NotFound(String),
}
pub type UserResult<T> = Result<T, UserServiceError>;

// Example database function that returns an opaque error type that may not implement your `derive`d traits
fn query_database() -> Result<(), sqlx::Error> {
    // Database operations...
    Err(sqlx::Error::RowNotFound)
}

// Example auth service with a different error type
fn verify_credentials() -> Result<(), oauth2::Error> {
    // Verification process...
    Err(oauth2::Error::ServerResponse("Invalid token".into()))
}

// Application code using domain errors with AnyError
fn authenticate_user(username: &str) -> UserResult<()> {
    // Convert sqlx::Error to AnyError
    query_database().map_err(|e| UserServiceError::Database(e.into()))?;

    // Convert oauth2::Error to AnyError
    verify_credentials().map_err(|e| UserServiceError::Authentication(e.into()))?;

    if username == "unknown" {
        return Err(UserServiceError::NotFound(username.to_string()));
    }

    Ok(())
}
```

## Serialization Example

```rust
// Errors are easily serializable for logging or API responses
let error = authenticate_user("unknown").unwrap_err();
let json = serde_json::to_string(&error).unwrap();

// Results in properly formatted JSON with full error chain context
```

## License

MIT
