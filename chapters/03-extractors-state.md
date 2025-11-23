# Chapter 3: Extractors & Application State

## Overview
Extractors are actix-web's type-safe way to extract information from HTTP requests. In this chapter, we'll explore all the built-in extractors and learn how to manage application state that can be shared across routes.

## Step 1: Understanding Extractors

Extractors implement the `FromRequest` trait and allow you to extract typed data from HTTP requests. Actix-web supports up to 12 extractors per handler function.

Reference: [Actix Web Extractors Documentation](https://actix.rs/docs/extractors)

## Step 2: Path Extractor

The `Path` extractor extracts information from the request's URL path. It's the same concept we saw in routing, but here we're focusing on how extractors work to pull this data into your handler function.

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse};

// Extract single path parameter
async fn get_user(path: web::Path<u32>) -> impl Responder {
    let user_id = path.into_inner();
    HttpResponse::Ok().json(format!("User ID: {}", user_id))
}

// Extract multiple path parameters as tuple
async fn get_user_post(path: web::Path<(u32, u32)>) -> impl Responder {
    let (user_id, post_id) = path.into_inner();
    HttpResponse::Ok().json(format!("User {} Post {}", user_id, post_id))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/users/{id}", web::get().to(get_user))
            .route("/users/{user_id}/posts/{post_id}", web::get().to(get_user_post))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 3: Path Extractor with Structs

Instead of using tuples, you can extract path parameters into named structs using serde. This approach is cleaner and more maintainable when you have multiple path parameters.

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

async fn get_user_post(path: web::Path<UserPostPath>) -> impl Responder {
    HttpResponse::Ok().json(format!("User {} Post {}", path.user_id, path.post_id))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/users/{id}", web::get().to(get_user))
            .route("/users/{user_id}/posts/{post_id}", web::get().to(get_user_post))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 4: Query Extractor

The `Query` extractor automatically parses URL query parameters (the part after `?` in the URL) into a Rust struct. This makes it easy to handle pagination, filtering, and other optional parameters.

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
struct Pagination {
    page: Option<u32>,
    limit: Option<u32>,
    sort: Option<String>,
}

async fn get_users(query: web::Query<Pagination>) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(10);
    let sort = query.sort.as_deref().unwrap_or("id");
    
    HttpResponse::Ok().json(format!(
        "Page: {}, Limit: {}, Sort: {}",
        page, limit, sort
    ))
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

## Step 5: JSON Extractor

The `Json` extractor automatically deserializes JSON request bodies into your Rust structs. This is the most common way to receive data from API clients.

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

## Step 6: JSON Extractor Configuration

You can configure the JSON extractor to set payload size limits and customize error handling. This helps protect your application from oversized requests and provides better error messages.

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
            .app_data(
                web::JsonConfig::default()
                    .limit(4096) // Limit JSON payload to 4KB
                    .error_handler(|err, _req| {
                        actix_web::error::InternalError::from_response(
                            err,
                            HttpResponse::BadRequest().json("Invalid JSON payload"),
                        )
                        .into()
                    })
            )
            .route("/users", web::post().to(create_user))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 7: Form Extractor

The `Form` extractor handles traditional HTML form submissions. It parses URL-encoded form data (the default format for HTML forms) into your Rust structs.

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

async fn login(form: web::Form<LoginForm>) -> impl Responder {
    HttpResponse::Ok().json(format!(
        "Login attempt for user: {}",
        form.username
    ))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(
                web::FormConfig::default()
                    .limit(1024) // Limit form size to 1KB
            )
            .route("/login", web::post().to(login))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 8: HttpRequest Extractor

The `HttpRequest` extractor gives you access to the entire HTTP request object. Use this when you need to inspect headers, the request method, or other request metadata that isn't covered by other extractors.

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse, HttpRequest};

async fn get_request_info(req: HttpRequest) -> impl Responder {
    let method = req.method();
    let path = req.path();
    let headers = req.headers();
    let user_agent = headers.get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("Unknown");
    
    HttpResponse::Ok().json(format!(
        "Method: {}, Path: {}, User-Agent: {}",
        method, path, user_agent
    ))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/info", web::get().to(get_request_info))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 9: String and Bytes Extractors

Sometimes you need the raw request body without any parsing. The `String` and `Bytes` extractors give you the body as-is, useful for custom parsing or when working with non-JSON formats.

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use actix_web::web::Bytes;

async fn echo_string(body: String) -> impl Responder {
    HttpResponse::Ok().body(format!("Received: {}", body))
}

async fn echo_bytes(body: Bytes) -> impl Responder {
    HttpResponse::Ok().body(format!("Received {} bytes", body.len()))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/echo/string", web::post().to(echo_string))
            .route("/echo/bytes", web::post().to(echo_bytes))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 10: Application State with web::Data

Application state allows you to share data across all routes using `web::Data`. This is perfect for database connections, configuration, or shared counters that need to be accessible from any handler. When your application runs with multiple worker threads, use `Arc: <Mutex<T>>` (Atomically Reference Counted) to share mutable state across all threads. This ensures all workers see the same shared data. 

Arc is for shared immutable data, and Mutex is needed when multiple threads need to mutate the same data simultaneously. 

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use std::sync::{Arc, Mutex};

struct AppState {
    counter: Arc<Mutex<i32>>,
    app_name: String,
}

async fn get_counter(data: web::Data<AppState>) -> impl Responder {
    let counter = data.counter.lock().unwrap();
    HttpResponse::Ok().json(format!(
        "App: {}, Counter: {}",
        data.app_name, *counter
    ))
}

async fn increment_counter(data: web::Data<AppState>) -> impl Responder {
    let mut counter = data.counter.lock().unwrap();
    *counter += 1;
    HttpResponse::Ok().json(format!("Counter: {}", *counter))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(AppState {
        counter: Arc::new(Mutex::new(0)),
        app_name: "Actix Tutorial".to_string(),
    });
    
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/counter", web::get().to(get_counter))
            .route("/counter/increment", web::post().to(increment_counter))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 11: Multiple Extractors in One Handler

You can combine multiple extractors in a single handler function (up to 12). This allows you to extract path parameters, query strings, request bodies, headers, and application state all at once.

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse, HttpRequest};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

async fn create_user(
    path: web::Path<u32>,
    query: web::Query<std::collections::HashMap<String, String>>,
    body: web::Json<CreateUser>,
    req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    let user_id = path.into_inner();
    let source = query.get("source").unwrap_or(&"unknown".to_string());
    let user_agent = req.headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("Unknown");
    
    HttpResponse::Created().json(format!(
        "Created user {} for account {} from {} using {}",
        body.name, user_id, source, user_agent
    ))
}

struct AppState {
    app_name: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(AppState {
        app_name: "Actix Tutorial".to_string(),
    });
    
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/accounts/{id}/users", web::post().to(create_user))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 12: Body Extractors - Important Note

**Important**: The request body can only be read once. If an extractor reads the request body, only the first such extractor will succeed. If you need fallback behavior (e.g., accept either JSON or form data), use the `Either` wrapper.

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use actix_web::web::Either;
use serde::Deserialize;

#[derive(Deserialize)]
struct User {
    name: String,
}

async fn create_user(
    body: Either<web::Json<User>, web::Form<User>>
) -> impl Responder {
    let user = match body {
        Either::Left(json) => json.into_inner(),
        Either::Right(form) => form.into_inner(),
    };
    
    HttpResponse::Created().json(user)
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

## Step 13: Custom Extractor

You can create your own extractors by implementing the `FromRequest` trait. This is useful for extracting custom data like API keys, authentication tokens, or any other request-specific information.

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse, FromRequest, HttpRequest};
use actix_web::dev::Payload;
use std::future::{ready, Ready};
use std::pin::Pin;

struct ApiKey(String);

impl FromRequest for ApiKey {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let api_key = req.headers()
            .get("X-API-Key")
            .and_then(|h| h.to_str().ok())
            .map(|s| ApiKey(s.to_string()));
        
        match api_key {
            Some(key) => ready(Ok(key)),
            None => ready(Err(actix_web::error::ErrorUnauthorized("Missing API key"))),
        }
    }
}

async fn protected_route(api_key: ApiKey) -> impl Responder {
    HttpResponse::Ok().json(format!("API Key: {}", api_key.0))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/protected", web::get().to(protected_route))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 14: Complete Example - All Extractors Together

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse, HttpRequest};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Deserialize, Serialize)]
struct CreateUser {
    name: String,
    email: String,
}

#[derive(Deserialize)]
struct Pagination {
    page: Option<u32>,
    limit: Option<u32>,
}

struct AppState {
    users: Arc<Mutex<Vec<String>>>,
}

async fn get_users(
    query: web::Query<Pagination>,
    data: web::Data<AppState>,
) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(10);
    let users = data.users.lock().unwrap();
    
    HttpResponse::Ok().json(format!(
        "Page: {}, Limit: {}, Users: {}",
        page, limit, users.len()
    ))
}

async fn create_user(
    path: web::Path<u32>,
    body: web::Json<CreateUser>,
    req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    let account_id = path.into_inner();
    let user_agent = req.headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("Unknown");
    
    let mut users = data.users.lock().unwrap();
    users.push(body.name.clone());
    
    HttpResponse::Created().json(format!(
        "Created user {} for account {} from {}",
        body.name, account_id, user_agent
    ))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(AppState {
        users: Arc::new(Mutex::new(Vec::new())),
    });
    
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/accounts/{id}/users", web::get().to(get_users))
            .route("/accounts/{id}/users", web::post().to(create_user))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Key Concepts Explained

- **Extractors**: Type-safe request information extraction via `FromRequest` trait
- **Path**: Extract URL path parameters
- **Query**: Extract URL query parameters
- **Json**: Deserialize JSON request body
- **Form**: Extract URL-encoded form data
- **HttpRequest**: Access the entire request object
- **String/Bytes**: Extract raw request body
- **web::Data**: Share application state across routes
- **Arc + Mutex**: Share mutable state across threads
- **Custom Extractors**: Implement `FromRequest` for custom extraction logic

## Important Notes

1. **Body Reading**: Only one extractor can read the request body. Use `Either` for fallback behavior.
2. **Extractor Limit**: Maximum of 12 extractors per handler function.
3. **State Sharing**: Use `web::Data` for read-only state, `Arc<Mutex<T>>` for shared mutable state.
4. **Thread Safety**: Be careful with blocking operations in async code when using `Mutex`.

## Best Practices

1. Use structs with serde for path and query parameters
2. Configure JSON/Form extractors with size limits
3. Use `Arc<Mutex<T>>` for state shared across all worker threads
4. Keep extractor logic simple and focused
5. Handle extraction errors gracefully

## Testing Extractors

```bash
# Test path extractor
curl http://127.0.0.1:8080/users/123

# Test query extractor
curl "http://127.0.0.1:8080/users?page=2&limit=20"

# Test JSON extractor
curl -X POST http://127.0.0.1:8080/users \
  -H "Content-Type: application/json" \
  -d '{"name":"Alice","email":"alice@example.com"}'

# Test form extractor
curl -X POST http://127.0.0.1:8080/login \
  -d "username=alice&password=secret"
```

## Next Steps
In the next chapter, we'll learn how to serve static files and integrate HTMX for dynamic frontend interactions.
