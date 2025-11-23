# Chapter 8: Error Handling & JSON Serialization

## Overview
Proper error handling is crucial for production applications. In this chapter, we'll learn how to create custom error types, implement error responses, and work with JSON serialization.

## Step 1: Update Cargo.toml

We'll use `thiserror` to simplify error type definitions and `serde_json` for JSON serialization. These crates make error handling more ergonomic and type-safe.

```toml
[dependencies]
thiserror = "1.0"
```

## Step 2: Custom Error Types

Custom error types allow you to create domain-specific errors that can be converted into HTTP responses. By implementing `ResponseError`, you control how errors are presented to clients.

```rust
use actix_web::{web, App, HttpServer, HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug)]
enum AppError {
    NotFound(String),
    BadRequest(String),
    InternalError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            AppError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::NotFound(_) => HttpResponse::NotFound().json(self),
            AppError::BadRequest(_) => HttpResponse::BadRequest().json(self),
            AppError::InternalError(_) => HttpResponse::InternalServerError().json(self),
        }
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

impl From<AppError> for ErrorResponse {
    fn from(err: AppError) -> Self {
        ErrorResponse {
            error: format!("{}", err),
            message: match err {
                AppError::NotFound(msg) => msg,
                AppError::BadRequest(msg) => msg,
                AppError::InternalError(msg) => msg,
            },
        }
    }
}

async fn get_user(id: web::Path<u32>) -> Result<HttpResponse, AppError> {
    if id.into_inner() == 0 {
        return Err(AppError::BadRequest("Invalid user ID".to_string()));
    }
    
    // Simulate not found
    Err(AppError::NotFound("User not found".to_string()))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/users/{id}", web::get().to(get_user))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 3: Using thiserror for Better Error Handling

The `thiserror` crate automatically generates error implementations, reducing boilerplate. It also makes it easy to wrap errors from other libraries and create error chains.

```rust
use actix_web::{web, App, HttpServer, HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Internal server error: {0}")]
    InternalError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let status = match self {
            AppError::NotFound(_) => actix_web::http::StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => actix_web::http::StatusCode::BAD_REQUEST,
            AppError::InternalError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::DatabaseError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        };
        
        HttpResponse::build(status).json(ErrorResponse {
            error: self.to_string(),
            message: format!("{}", self),
        })
    }
}

async fn get_user(id: web::Path<u32>) -> Result<HttpResponse, AppError> {
    let user_id = id.into_inner();
    
    if user_id == 0 {
        return Err(AppError::BadRequest("Invalid user ID".to_string()));
    }
    
    // Simulate database query
    if user_id > 100 {
        return Err(AppError::NotFound(format!("User {} not found", user_id)));
    }
    
    Ok(HttpResponse::Ok().json(format!("User {}", user_id)))
}
```

## Step 4: JSON Serialization Best Practices

Follow best practices when serializing data to JSON: skip sensitive fields like passwords, use consistent response structures, and validate input before processing.

```rust
use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct User {
    id: u32,
    name: String,
    email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<String>, // Never serialize passwords!
    #[serde(default)]
    active: bool,
}

#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: T,
}

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    error: String,
}

async fn get_user() -> impl Responder {
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        password: Some("secret".to_string()),
        active: true,
    };
    
    // Password will be skipped in serialization
    HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: user,
    })
}

async fn create_user(user: web::Json<User>) -> impl Responder {
    // Validate input
    if user.email.is_empty() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            error: "Email is required".to_string(),
        });
    }
    
    HttpResponse::Created().json(ApiResponse {
        success: true,
        data: user.into_inner(),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/users", web::get().to(get_user))
            .route("/users", web::post().to(create_user))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 5: Custom JSON Configuration

You can configure the JSON extractor to set size limits and customize error handling. This helps protect your application from oversized payloads and provides better error messages.

```rust
use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
}

async fn get_user() -> impl Responder {
    let user = User {
        id: 1,
        name: "Alice".to_string(),
    };
    
    // Custom JSON serialization
    let json = serde_json::json!({
        "user": user,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });
    
    HttpResponse::Ok().json(json)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(
                web::JsonConfig::default()
                    .limit(4096) // Limit JSON payload size
                    .error_handler(|err, _req| {
                        actix_web::error::InternalError::from_response(
                            err,
                            HttpResponse::BadRequest().json("Invalid JSON"),
                        )
                        .into()
                    })
            )
            .route("/users", web::get().to(get_user))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 6: Error Handling Middleware

Error handling middleware can intercept error responses and add custom headers or modify error messages. This ensures consistent error formatting across your entire application.

```rust
use actix_web::{web, App, HttpServer, HttpResponse, ResponseError, middleware::ErrorHandlers};
use serde::Serialize;

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    code: u16,
}

fn add_error_header<B>(
    mut res: actix_web::dev::ServiceResponse<B>,
) -> Result<actix_web::dev::ServiceResponse<B>, actix_web::Error> {
    let status = res.status();
    
    if status.is_client_error() || status.is_server_error() {
        let error_response = ErrorResponse {
            error: status.canonical_reason().unwrap_or("Unknown error").to_string(),
            code: status.as_u16(),
        };
        
        res.response_mut().headers_mut().insert(
            actix_web::http::header::HeaderName::from_static("x-error"),
            actix_web::http::HeaderValue::from_static("true"),
        );
    }
    
    Ok(res)
}

async fn error_handler() -> Result<HttpResponse, actix_web::Error> {
    Err(actix_web::error::ErrorInternalServerError("Something went wrong"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(
                ErrorHandlers::new()
                    .handler(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, add_error_header)
                    .handler(actix_web::http::StatusCode::NOT_FOUND, add_error_header)
            )
            .route("/error", web::get().to(error_handler))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 7: Validation with serde

Input validation ensures data integrity before processing. Use the `validator` crate to add validation rules directly to your structs, making validation declarative and easy to maintain.

```rust
use actix_web::{web, App, HttpServer, HttpResponse, Responder, Result};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

#[derive(Deserialize, Validate)]
struct CreateUser {
    #[validate(length(min = 3, max = 50))]
    name: String,
    
    #[validate(email)]
    email: String,
    
    #[validate(range(min = 18, max = 120))]
    age: u32,
}

async fn create_user(user: web::Json<CreateUser>) -> Result<HttpResponse> {
    // Validate the input
    user.validate()
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Validation error: {}", e)))?;
    
    Ok(HttpResponse::Created().json(user.into_inner()))
}
```

## Step 8: Complete Example with Error Handling

This example combines all error handling concepts: custom error types, JSON serialization, validation, and consistent error responses. It demonstrates a production-ready error handling strategy.

```rust
use actix_web::{web, App, HttpServer, HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Internal server error")]
    InternalError,
}

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    error: String,
    message: String,
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let status = match self {
            AppError::NotFound(_) => actix_web::http::StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => actix_web::http::StatusCode::BAD_REQUEST,
            AppError::InternalError => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        };
        
        HttpResponse::build(status).json(ErrorResponse {
            success: false,
            error: format!("{}", self),
            message: self.to_string(),
        })
    }
}

#[derive(Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: T,
}

async fn get_user(id: web::Path<u32>) -> Result<HttpResponse, AppError> {
    let user_id = id.into_inner();
    
    if user_id == 0 {
        return Err(AppError::BadRequest("Invalid user ID".to_string()));
    }
    
    if user_id > 100 {
        return Err(AppError::NotFound(format!("User {} not found", user_id)));
    }
    
    let user = User {
        id: user_id,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    
    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: user,
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(
                web::JsonConfig::default()
                    .limit(4096)
            )
            .route("/users/{id}", web::get().to(get_user))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Key Concepts Explained

- **ResponseError Trait**: Implement this to create custom error types
- **thiserror**: Makes error handling easier and more ergonomic
- **JSON Serialization**: Use serde for type-safe serialization
- **Error Middleware**: Customize error responses globally
- **Validation**: Validate input before processing

## Best Practices

1. Always implement `ResponseError` for custom errors
2. Use `thiserror` for cleaner error definitions
3. Never serialize sensitive data (passwords, tokens)
4. Provide meaningful error messages
5. Use appropriate HTTP status codes
6. Log errors for debugging
7. Validate input early

## Common Error Patterns

- **NotFound**: 404 - Resource doesn't exist
- **BadRequest**: 400 - Invalid input
- **Unauthorized**: 401 - Not authenticated
- **Forbidden**: 403 - Not authorized
- **InternalServerError**: 500 - Server error

## Next Steps
In the final chapter, we'll cover testing and best practices for production applications.

