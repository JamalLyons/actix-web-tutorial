# Chapter 7: Database Integration with actix-web

## Overview
In this chapter, we'll learn how to integrate databases with actix-web. We'll cover how to share database connections across handlers, use database pools in application state, and handle database operations within route handlers.

## Step 1: Update Cargo.toml

We'll use `sqlx` for database operations, which provides compile-time checked SQL queries and connection pooling. The `runtime-tokio-rustls` feature enables async runtime support.

```toml
[dependencies]
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "sqlite", "migrate"] }
```

## Step 2: Sharing Database Connections with Application State

In actix-web, you share database connections across all handlers using `web::Data`. This allows multiple concurrent requests to efficiently share a connection pool.

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse, Result};
use sqlx::{SqlitePool, SqlitePoolOptions};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: i64,
    name: String,
    email: String,
}

// Handler receives database pool via web::Data extractor
async fn get_users(pool: web::Data<SqlitePool>) -> Result<HttpResponse> {
    let users = sqlx::query_as::<_, User>(
        "SELECT id, name, email FROM users"
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(users))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Create database connection pool
    let database_url = "sqlite:./tutorial.db";
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .expect("Failed to connect to database");
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    
    HttpServer::new(move || {
        App::new()
            // Share database pool with all handlers
            .app_data(web::Data::new(pool.clone()))
            .route("/users", web::get().to(get_users))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

**Key Points:**
- `web::Data<T>` is an extractor that provides shared, read-only access to data
- The pool is cloned for each worker thread, but connections are shared efficiently
- All handlers can access the same database pool through the `web::Data` extractor

## Step 3: Database Operations in Handlers

Handlers can combine database operations with other extractors. Here's how to perform CRUD operations:

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse, Result};
use sqlx::{SqlitePool, FromRow};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, FromRow)]
struct User {
    id: i64,
    name: String,
    email: String,
}

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

// Read: Get all users
async fn get_users(pool: web::Data<SqlitePool>) -> Result<HttpResponse> {
    let users = sqlx::query_as::<_, User>(
        "SELECT id, name, email FROM users ORDER BY created_at DESC"
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(users))
}

// Read: Get user by ID (combining Path and Data extractors)
async fn get_user(
    path: web::Path<i64>,
    pool: web::Data<SqlitePool>,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();
    
    let user = sqlx::query_as::<_, User>(
        "SELECT id, name, email FROM users WHERE id = ?"
    )
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    match user {
        Some(user) => Ok(HttpResponse::Ok().json(user)),
        None => Ok(HttpResponse::NotFound().json("User not found")),
    }
}

// Create: Combine Json and Data extractors
async fn create_user(
    user: web::Json<CreateUser>,
    pool: web::Data<SqlitePool>,
) -> Result<HttpResponse> {
    let result = sqlx::query(
        "INSERT INTO users (name, email) VALUES (?, ?)"
    )
    .bind(&user.name)
    .bind(&user.email)
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE constraint") {
            actix_web::error::ErrorConflict("Email already exists")
        } else {
            actix_web::error::ErrorInternalServerError(e)
        }
    })?;
    
    let created_user = sqlx::query_as::<_, User>(
        "SELECT id, name, email FROM users WHERE id = ?"
    )
    .bind(result.last_insert_rowid())
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Created().json(created_user))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:./tutorial.db")
        .await
        .expect("Failed to connect to database");
    
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/users", web::get().to(get_users))
            .route("/users/{id}", web::get().to(get_user))
            .route("/users", web::post().to(create_user))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

**Key Points:**
- You can combine multiple extractors: `web::Path`, `web::Json`, `web::Data`, etc.
- Database operations are async and use `.await`
- Convert database errors to HTTP errors using `map_err()`

## Step 4: Update and Delete Operations

Update and delete operations follow the same pattern, combining extractors with database operations:

```rust
#[derive(Deserialize)]
struct UpdateUser {
    name: Option<String>,
    email: Option<String>,
}

// Update: Combine Path, Json, and Data extractors
async fn update_user(
    path: web::Path<i64>,
    user: web::Json<UpdateUser>,
    pool: web::Data<SqlitePool>,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();
    
    // Check if user exists
    let current = sqlx::query_as::<_, (String, String)>(
        "SELECT name, email FROM users WHERE id = ?"
    )
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    let (current_name, current_email) = match current {
        Some((name, email)) => (name, email),
        None => return Ok(HttpResponse::NotFound().json("User not found")),
    };
    
    // Use provided values or keep existing ones
    let name = user.name.as_ref().unwrap_or(&current_name);
    let email = user.email.as_ref().unwrap_or(&current_email);
    
    sqlx::query("UPDATE users SET name = ?, email = ? WHERE id = ?")
        .bind(name)
        .bind(email)
        .bind(user_id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    let updated_user = sqlx::query_as::<_, User>(
        "SELECT id, name, email FROM users WHERE id = ?"
    )
    .bind(user_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(updated_user))
}

// Delete: Combine Path and Data extractors
async fn delete_user(
    path: web::Path<i64>,
    pool: web::Data<SqlitePool>,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();
    
    let result = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(user_id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    if result.rows_affected() == 0 {
        Ok(HttpResponse::NotFound().json("User not found"))
    } else {
        Ok(HttpResponse::Ok().json("User deleted"))
    }
}
```

## Step 5: Combining Database with Application State

You can combine database pools with other application state. This is useful when you need both database access and other shared resources:

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse, Result};
use sqlx::SqlitePool;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    request_count: Arc<Mutex<u64>>,
}

async fn get_users(state: web::Data<AppState>) -> Result<HttpResponse> {
    // Access database from state
    let users = sqlx::query_as::<_, User>(
        "SELECT id, name, email FROM users"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    // Also access other shared state
    let mut count = state.request_count.lock().unwrap();
    *count += 1;
    
    Ok(HttpResponse::Ok().json(users))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:./tutorial.db")
        .await
        .expect("Failed to connect to database");
    
    let app_state = web::Data::new(AppState {
        db: pool,
        request_count: Arc::new(Mutex::new(0)),
    });
    
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/users", web::get().to(get_users))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 6: Error Handling with Database Operations

Proper error handling is crucial when working with databases. Convert database errors to appropriate HTTP responses:

```rust
async fn create_user(
    user: web::Json<CreateUser>,
    pool: web::Data<SqlitePool>,
) -> Result<HttpResponse> {
    let result = sqlx::query(
        "INSERT INTO users (name, email) VALUES (?, ?)"
    )
    .bind(&user.name)
    .bind(&user.email)
    .execute(pool.get_ref())
    .await;
    
    match result {
        Ok(insert_result) => {
            let created_user = sqlx::query_as::<_, User>(
                "SELECT id, name, email FROM users WHERE id = ?"
            )
            .bind(insert_result.last_insert_rowid())
            .fetch_one(pool.get_ref())
            .await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
            
            Ok(HttpResponse::Created().json(created_user))
        }
        Err(e) => {
            // Handle specific database errors
            if e.to_string().contains("UNIQUE constraint") {
                Err(actix_web::error::ErrorConflict("Email already exists"))
            } else if e.to_string().contains("NOT NULL constraint") {
                Err(actix_web::error::ErrorBadRequest("Missing required field"))
            } else {
                Err(actix_web::error::ErrorInternalServerError(e))
            }
        }
    }
}
```

## Step 7: Database Migrations

Migrations are SQL scripts that set up or modify your database schema. They're versioned and run automatically when your application starts.

Create `migrations/001_create_users.sql`:

```sql
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
```

**Optional: Seed Data Migration**

You can also create a seed migration to populate your database with sample data for testing:

Create `migrations/002_seed_users.sql`:

```sql
-- Seed sample users for tutorial
INSERT INTO users (name, email, created_at) VALUES
    ('Alice Johnson', 'alice@example.com', datetime('now', '-7 days')),
    ('Bob Smith', 'bob@example.com', datetime('now', '-5 days')),
    ('Charlie Brown', 'charlie@example.com', datetime('now', '-3 days')),
    ('Diana Prince', 'diana@example.com', datetime('now', '-2 days')),
    ('Eve Wilson', 'eve@example.com', datetime('now', '-1 day'))
ON CONFLICT(email) DO NOTHING;
```

The `ON CONFLICT(email) DO NOTHING` clause ensures the seed data won't cause errors if you run migrations multiple times.

Run migrations in your main function:

```rust
sqlx::migrate!("./migrations")
    .run(&pool)
    .await
    .expect("Failed to run migrations");
```

## Step 8: Complete Example

Here's a complete example showing all CRUD operations with proper actix-web integration:

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse, Result};
use sqlx::{SqlitePool, SqlitePoolOptions, FromRow};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, FromRow)]
struct User {
    id: i64,
    name: String,
    email: String,
}

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

#[derive(Deserialize)]
struct UpdateUser {
    name: Option<String>,
    email: Option<String>,
}

async fn get_users(pool: web::Data<SqlitePool>) -> Result<HttpResponse> {
    let users = sqlx::query_as::<_, User>(
        "SELECT id, name, email FROM users ORDER BY created_at DESC"
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(users))
}

async fn get_user(
    path: web::Path<i64>,
    pool: web::Data<SqlitePool>,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();
    
    let user = sqlx::query_as::<_, User>(
        "SELECT id, name, email FROM users WHERE id = ?"
    )
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    match user {
        Some(user) => Ok(HttpResponse::Ok().json(user)),
        None => Ok(HttpResponse::NotFound().json("User not found")),
    }
}

async fn create_user(
    user: web::Json<CreateUser>,
    pool: web::Data<SqlitePool>,
) -> Result<HttpResponse> {
    let result = sqlx::query(
        "INSERT INTO users (name, email) VALUES (?, ?)"
    )
    .bind(&user.name)
    .bind(&user.email)
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE constraint") {
            actix_web::error::ErrorConflict("Email already exists")
        } else {
            actix_web::error::ErrorInternalServerError(e)
        }
    })?;
    
    let created_user = sqlx::query_as::<_, User>(
        "SELECT id, name, email FROM users WHERE id = ?"
    )
    .bind(result.last_insert_rowid())
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Created().json(created_user))
}

async fn update_user(
    path: web::Path<i64>,
    user: web::Json<UpdateUser>,
    pool: web::Data<SqlitePool>,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();
    
    let current = sqlx::query_as::<_, (String, String)>(
        "SELECT name, email FROM users WHERE id = ?"
    )
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    let (current_name, current_email) = match current {
        Some((name, email)) => (name, email),
        None => return Ok(HttpResponse::NotFound().json("User not found")),
    };
    
    let name = user.name.as_ref().unwrap_or(&current_name);
    let email = user.email.as_ref().unwrap_or(&current_email);
    
    sqlx::query("UPDATE users SET name = ?, email = ? WHERE id = ?")
        .bind(name)
        .bind(email)
        .bind(user_id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    let updated_user = sqlx::query_as::<_, User>(
        "SELECT id, name, email FROM users WHERE id = ?"
    )
    .bind(user_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(updated_user))
}

async fn delete_user(
    path: web::Path<i64>,
    pool: web::Data<SqlitePool>,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();
    
    let result = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(user_id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    if result.rows_affected() == 0 {
        Ok(HttpResponse::NotFound().json("User not found"))
    } else {
        Ok(HttpResponse::Ok().json("User deleted"))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:./tutorial.db")
        .await
        .expect("Failed to connect to database");
    
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
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

- **`web::Data<T>`**: Extractor that provides shared access to data across all handlers
- **Connection Pooling**: Efficiently share database connections across concurrent requests
- **Combining Extractors**: Use multiple extractors together (Path, Json, Data, etc.)
- **Error Handling**: Convert database errors to appropriate HTTP responses
- **Async Operations**: All database operations use `.await` for async execution

## Best Practices

1. **Use `web::Data` for database pools**: Share connections efficiently across handlers
2. **Combine extractors**: Use Path, Json, Query, and Data extractors together
3. **Handle errors properly**: Convert database errors to HTTP status codes
4. **Use connection pools**: Never create new connections in handlers
5. **Run migrations on startup**: Ensure database schema is up to date

## Testing Database Operations

You can test with curl:

```bash
# Create user
curl -X POST http://127.0.0.1:8080/users \
  -H "Content-Type: application/json" \
  -d '{"name":"Alice","email":"alice@example.com"}'

# Get all users
curl http://127.0.0.1:8080/users

# Get user by ID
curl http://127.0.0.1:8080/users/1

# Update user
curl -X PUT http://127.0.0.1:8080/users/1 \
  -H "Content-Type: application/json" \
  -d '{"name":"Alice Updated","email":"alice.updated@example.com"}'

# Delete user
curl -X DELETE http://127.0.0.1:8080/users/1
```

## Next Steps
In the next chapter, we'll cover error handling and JSON serialization in more detail.
