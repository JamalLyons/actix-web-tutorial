# Chapter 9: Testing & Best Practices

## Overview
In this final chapter, we'll learn how to write tests for actix-web applications and cover best practices for production-ready code.


## Step 1: Basic Unit Tests

Unit tests verify individual functions work correctly. Actix-web provides testing utilities that let you test handlers without starting a full HTTP server, making tests fast and reliable.

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse};

async fn hello() -> impl Responder {
    "Hello, World!"
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_hello() {
        let app = test::init_service(
            App::new().route("/", web::get().to(hello))
        ).await;
        
        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.status().is_success());
    }
}
```

## Step 2: Integration Tests

Integration tests verify that multiple components work together correctly. They test the full request/response cycle, including extractors, routing, and JSON serialization.

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
}

async fn get_user(id: web::Path<u32>) -> impl Responder {
    HttpResponse::Ok().json(User {
        id: id.into_inner(),
        name: "Alice".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};

    #[actix_web::test]
    async fn test_get_user() {
        let app = test::init_service(
            App::new().route("/users/{id}", web::get().to(get_user))
        ).await;
        
        let req = test::TestRequest::get()
            .uri("/users/1")
            .to_request();
        
        let resp: User = test::call_and_read_body_json(&app, req).await;
        
        assert_eq!(resp.id, 1);
        assert_eq!(resp.name, "Alice");
    }
}
```

## Step 3: Testing with Database

When testing database operations, use a separate test database or in-memory database. This ensures your tests don't affect production data and can run in parallel safely.

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use sqlx::{SqlitePool, FromRow};
use serde::Serialize;

#[derive(Serialize, FromRow)]
struct User {
    id: i64,
    name: String,
}

async fn get_users(pool: web::Data<SqlitePool>) -> impl Responder {
    let users = sqlx::query_as::<_, User>("SELECT id, name FROM users")
        .fetch_all(pool.get_ref())
        .await
        .unwrap();
    
    HttpResponse::Ok().json(users)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};
    use sqlx::sqlite::SqlitePoolOptions;

    #[actix_web::test]
    async fn test_get_users() {
        // Use in-memory database for tests
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();
        
        // Setup test data
        sqlx::query("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)")
            .execute(&pool)
            .await
            .unwrap();
        
        sqlx::query("INSERT INTO users (name) VALUES ('Alice')")
            .execute(&pool)
            .await
            .unwrap();
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .route("/users", web::get().to(get_users))
        ).await;
        
        let req = test::TestRequest::get().uri("/users").to_request();
        let resp: Vec<User> = test::call_and_read_body_json(&app, req).await;
        
        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].name, "Alice");
    }
}
```

## Step 4: Best Practices - Configuration

Store configuration in environment variables rather than hardcoding values. This makes your application flexible across different environments (development, staging, production) and keeps secrets out of your code.

```rust
use actix_web::{web, App, HttpServer};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
struct Config {
    host: String,
    port: u16,
    database_url: String,
}

impl Config {
    fn from_env() -> Self {
        Config {
            host: std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:./app.db".to_string()),
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Config::from_env();
    
    HttpServer::new(|| {
        App::new()
    })
    .bind(format!("{}:{}", config.host, config.port))?
    .run()
    .await
}
```

## Step 5: Best Practices - Logging

Proper logging helps you debug issues and monitor your application in production. Use structured logging with appropriate log levels (error, warn, info, debug) to track what's happening in your application.

```rust
use actix_web::{web, App, HttpServer, middleware::Logger};
use env_logger;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(
        env_logger::Env::default().default_filter_or("info")
    );
    
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 6: Best Practices - Graceful Shutdown

Graceful shutdown ensures your application finishes processing current requests before stopping. This prevents data loss and provides a better user experience when deploying updates.

```rust
use actix_web::{web, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let server = HttpServer::new(|| {
        App::new()
    })
    .bind("127.0.0.1:8080")?
    .run();
    
    // Handle shutdown signals
    let server_handle = server.handle();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        println!("Shutting down...");
        server_handle.stop(true).await;
    });
    
    server.await
}
```