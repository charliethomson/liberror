use liberror::AnyError;
use serde::{Deserialize, Serialize};
use std::error::Error;
use thiserror::Error;

/// Simulate an external crate with its own error type
#[allow(unused)]
mod external_db {
    #[derive(Debug)]
    pub enum DatabaseError {
        ConnectionFailed,
        QueryFailed,
        RowNotFound,
    }

    impl std::fmt::Display for DatabaseError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::ConnectionFailed => write!(f, "Database connection failed"),
                Self::QueryFailed => write!(f, "Query execution failed"),
                Self::RowNotFound => write!(f, "Row not found"),
            }
        }
    }

    impl std::error::Error for DatabaseError {}

    pub fn query_database() -> Result<(), DatabaseError> {
        // Simulate a database query failure
        Err(DatabaseError::RowNotFound)
    }
}

/// Simulate another external crate with a different error type
#[allow(unused)]
mod auth_service {
    #[derive(Debug)]
    pub struct AuthError {
        pub message: String,
    }

    impl std::fmt::Display for AuthError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Authentication error: {}", self.message)
        }
    }

    impl std::error::Error for AuthError {}

    pub fn verify_credentials() -> Result<(), AuthError> {
        // Simulate an authentication error
        Err(AuthError {
            message: "Invalid token".to_string(),
        })
    }
}

// Define our application error enum using thiserror
#[derive(Debug, Clone, Serialize, Deserialize, Error)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "$type",
    content = "context"
)]
pub enum UserServiceError {
    #[error("Database error: {0}")]
    #[serde(rename = "app.service.user.database")]
    Database(AnyError),

    #[error("Authentication error: {0}")]
    #[serde(rename = "app.service.user.auth")]
    Authentication(AnyError),

    #[error("User not found: {0}")]
    #[serde(rename = "app.service.user.not_found")]
    NotFound(String),
}

pub type UserResult<T> = Result<T, UserServiceError>;

// Application code using domain errors with AnyError
fn authenticate_user(username: &str) -> UserResult<()> {
    // Convert external DatabaseError to AnyError
    external_db::query_database().map_err(|e| UserServiceError::Database(e.into()))?;

    // Convert external AuthError to AnyError
    auth_service::verify_credentials().map_err(|e| UserServiceError::Authentication(e.into()))?;

    if username == "unknown" {
        return Err(UserServiceError::NotFound(username.to_string()));
    }

    Ok(())
}

// Create a nested error example to demonstrate error chain preservation
fn create_nested_error() -> UserResult<()> {
    // First error (not found)
    authenticate_user("unknown")
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("AnyError Example\n");

    // Example 1: Basic error handling
    println!("Example 1: Basic error handling");
    match authenticate_user("valid_user") {
        Ok(_) => println!("User authenticated successfully"),
        Err(e) => println!("Error: {}", e),
    }
    println!();

    // Example 2: Error with user not found
    println!("Example 2: Error with user not found");
    let result = authenticate_user("unknown");
    match &result {
        Ok(_) => println!("User authenticated successfully"),
        Err(e) => println!("Error: {}", e),
    }
    println!();

    // Example 3: Serialization
    println!("Example 3: Error serialization");
    if let Err(error) = result {
        let json = serde_json::to_string_pretty(&error)?;
        println!("Serialized error JSON:\n{}", json);
    }
    println!();

    // Example 4: Debug representation
    println!("Example 4: Debug representation");
    if let Err(error) = authenticate_user("test_user") {
        println!("Debug output:\n{:#?}", error);
    }
    println!();

    // Example 5: Nested errors
    println!("Example 5: Nested errors");
    if let Err(error) = create_nested_error() {
        println!("Nested error: {}", error);
        let json = serde_json::to_string_pretty(&error)?;
        println!("Nested error JSON:\n{}", json);
    }

    Ok(())
}
