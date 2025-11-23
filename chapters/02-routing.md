# Chapter 2: Routing

## Overview
Routing is fundamental to web applications. In this chapter, we'll learn how to handle different HTTP methods, path parameters, query parameters, and request bodies.

## Step 1: Update Cargo.toml

We need to add serde for serialization/deserialization, which allows us to work with JSON and other data formats in our routes.

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
```

## Step 2: Basic Routing with Different HTTP Methods

The same URL path can handle different HTTP methods (GET, POST, PUT, DELETE). Each method typically represents a different action: GET for reading, POST for creating, PUT for updating, and DELETE for removing resources.

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse};

async fn get_users() -> impl Responder {
    HttpResponse::Ok().json(vec!["Alice", "Bob", "Charlie"])
}

async fn create_user() -> impl Responder {
    HttpResponse::Created().json("User created")
}

async fn update_user() -> impl Responder {
    HttpResponse::Ok().json("User updated")
}

async fn delete_user() -> impl Responder {
    HttpResponse::Ok().json("User deleted")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/users", web::get().to(get_users))
            .route("/users", web::post().to(create_user))
            .route("/users", web::put().to(update_user))
            .route("/users", web::delete().to(delete_user))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 3: Path Parameters

Path parameters allow you to extract values from the URL path itself. For example, in `/users/{id}`, the `{id}` part is a path parameter that you can extract and use in your handler.

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse};

async fn get_user(path: web::Path<u32>) -> impl Responder {
    let user_id = path.into_inner();
    HttpResponse::Ok().json(format!("User ID: {}", user_id))
}

async fn get_user_posts(path: web::Path<(u32, u32)>) -> impl Responder {
    let (user_id, post_id) = path.into_inner();
    HttpResponse::Ok().json(format!("User {} Post {}", user_id, post_id))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/users/{id}", web::get().to(get_user))
            .route("/users/{user_id}/posts/{post_id}", web::get().to(get_user_posts))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 4: Using Structs for Path Parameters

Instead of using tuples, you can extract path parameters into named structs. This makes your code more readable and type-safe, especially when dealing with multiple parameters.

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
struct UserPath {
    id: u32,
}

#[derive(Deserialize)]
struct UserPostPath {
    user_id: u32,
    post_id: u32,
}

async fn get_user(path: web::Path<UserPath>) -> impl Responder {
    HttpResponse::Ok().json(format!("User ID: {}", path.id))
}

async fn get_user_posts(path: web::Path<UserPostPath>) -> impl Responder {
    HttpResponse::Ok().json(format!("User {} Post {}", path.user_id, path.post_id))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/users/{id}", web::get().to(get_user))
            .route("/users/{user_id}/posts/{post_id}", web::get().to(get_user_posts))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 5: Query Parameters

Query parameters are the key-value pairs that appear after the `?` in a URL (e.g., `?page=1&limit=10`). They're commonly used for filtering, pagination, and optional configuration.

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
struct Pagination {
    page: Option<u32>,
    limit: Option<u32>,
}

async fn get_users(query: web::Query<Pagination>) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(10);
    HttpResponse::Ok().json(format!("Page: {}, Limit: {}", page, limit))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/users", web::get().to(get_users))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 6: Request Bodies (JSON)

When clients send data in the request body (like when creating a new user), you can extract it as JSON. The `Json` extractor automatically deserializes the JSON into your Rust struct.

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct CreateUser {
    name: String,
    email: String,
}

async fn create_user(user: web::Json<CreateUser>) -> impl Responder {
    HttpResponse::Created().json(user.into_inner())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/users", web::post().to(create_user))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 7: Complete Example with All Routing Features

This example combines all the routing concepts: different HTTP methods, path parameters, query parameters, and JSON request bodies. Together, these form a complete CRUD (Create, Read, Update, Delete) API.

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

#[derive(Deserialize)]
struct Pagination {
    page: Option<u32>,
    limit: Option<u32>,
}

async fn get_users(query: web::Query<Pagination>) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(10);
    HttpResponse::Ok().json(format!("Fetching users: page={}, limit={}", page, limit))
}

async fn get_user(path: web::Path<u32>) -> impl Responder {
    let user_id = path.into_inner();
    let user = User {
        id: user_id,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    HttpResponse::Ok().json(user)
}

async fn create_user(user: web::Json<CreateUser>) -> impl Responder {
    let new_user = User {
        id: 1,
        name: user.name.clone(),
        email: user.email.clone(),
    };
    HttpResponse::Created().json(new_user)
}

#[derive(Deserialize)]
struct UpdateUser {
    name: Option<String>,
    email: Option<String>,
}

async fn update_user(path: web::Path<u32>, user: web::Json<UpdateUser>) -> impl Responder {
    let user_id = path.into_inner();
    let updated_user = User {
        id: user_id,
        name: user.name.clone().unwrap_or_else(|| "Unknown".to_string()),
        email: user.email.clone().unwrap_or_else(|| "unknown@example.com".to_string()),
    };
    HttpResponse::Ok().json(updated_user)
}

async fn delete_user(path: web::Path<u32>) -> impl Responder {
    let user_id = path.into_inner();
    HttpResponse::Ok().json(format!("User {} deleted", user_id))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/users", web::get().to(get_users))
            .route("/users/{id}", web::get().to(get_user))
            .route("/users", web::post().to(create_user))
            .route("/users/{id}", web::put().to(update_user))
            .route("/users/{id}", web::delete().to(delete_user))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Key Concepts Explained

- **HTTP Methods**: `web::get()`, `web::post()`, `web::put()`, `web::delete()`, etc.
- **Path Parameters**: Extract from URL path using `web::Path<T>`
- **Query Parameters**: Extract from URL query string using `web::Query<T>`
- **Request Bodies**: Extract JSON using `web::Json<T>`
- **Serde**: Used for serialization/deserialization of data structures
- **CRUD Operations**: Create (POST), Read (GET), Update (PUT), Delete (DELETE)

## Testing Your Routes

You can test these routes using:
- Browser (for GET requests)
- `curl`:
  ```bash
  curl http://127.0.0.1:8080/users
  curl http://127.0.0.1:8080/users/1
  curl -X POST http://127.0.0.1:8080/users \
    -H "Content-Type: application/json" \
    -d '{"name":"Alice","email":"alice@example.com"}'
  
  # Update user
  curl -X PUT http://127.0.0.1:8080/users/1 \
    -H "Content-Type: application/json" \
    -d '{"name":"Alice Updated","email":"alice.updated@example.com"}'
  
  # Delete user
  curl -X DELETE http://127.0.0.1:8080/users/1
  ```

## Next Steps
In the next chapter, we'll dive deep into extractors and learn how to manage application state that can be shared across routes.
