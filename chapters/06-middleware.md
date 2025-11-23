# Chapter 6: Middleware

## Overview
Middleware in actix-web allows you to process requests and responses at different stages of the request lifecycle. Middleware can hook into incoming request processing, enabling you to modify requests, halt processing to return early responses, and post-process responses.

Reference: [Actix Web Middleware Documentation](https://actix.rs/docs/middleware)

## Understanding Middleware

Middleware sits between the HTTP server and your route handlers. It can:

- **Pre-process the Request**: Modify or inspect the request before it reaches your handler
- **Post-process the Response**: Modify the response after your handler returns
- **Modify Application State**: Access and modify shared application state
- **Access External Services**: Connect to Redis, logging services, etc.
- **Halt Processing**: Return a response early without calling the handler

### Request/Response Lifecycle

```
HTTP Request
    ↓
Middleware 1 (pre-process)
    ↓
Middleware 2 (pre-process)
    ↓
Middleware 3 (pre-process)
    ↓
Route Handler (your code)
    ↓
Middleware 3 (post-process)
    ↓
Middleware 2 (post-process)
    ↓
Middleware 1 (post-process)
    ↓
HTTP Response
```

**Important**: Middleware is registered for each `App`, `scope`, or `Resource` and executed in **opposite order** as registration. The last middleware you wrap will be the first to execute.

## Step 1: Update Cargo.toml

We'll need `actix-cors` for handling cross-origin requests, `env_logger` for logging, and `actix-service` for creating custom middleware.

```toml
[dependencies]
actix-cors = "0.7"
env_logger = "0.11"
log = "0.4"
```

## Step 2: Logging Middleware

The `Logger` middleware automatically logs information about each HTTP request and response. It's one of the most commonly used built-in middleware, helping you debug and monitor your application.

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse, middleware::Logger};
use env_logger;

async fn hello() -> impl Responder {
    "Hello, World!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .route("/", web::get().to(hello))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

The default format logs: `%a %t "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T`

## Step 3: Simple Middleware with `wrap_fn`

For simple use cases, you can use `wrap_fn` to create small, ad-hoc middleware without implementing the full `Transform` and `Service` traits. This is perfect for quick logging or simple request/response modifications.

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse, middleware::Logger};

async fn hello() -> impl Responder {
    "Hello, World!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap_fn(|req, srv| {
                println!("Request: {} {}", req.method(), req.path());
                let fut = srv.call(req);
                async move {
                    let res = fut.await?;
                    println!("Response status: {}", res.status());
                    Ok(res)
                }
            })
            .route("/", web::get().to(hello))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 4: Using `from_fn` for Function-Based Middleware

The `from_fn` function converts a simple async function into middleware. This is cleaner than `wrap_fn` when you want to extract your middleware logic into a separate function.

You can also use `from_fn` in combination with `wrap` to create middleware from a function:

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse, dev::ServiceRequest};
use actix_web_lab::middleware::from_fn;

async fn log_request(req: ServiceRequest) -> Result<ServiceRequest, actix_web::Error> {
    println!("[LOG] {} {}", req.method(), req.path());
    Ok(req)
}

async fn hello() -> impl Responder {
    "Hello, World!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(from_fn(log_request))
            .route("/", web::get().to(hello))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 5: Custom Middleware - Understanding Transform and Service

For more complex middleware, you need to implement the `Transform` and `Service` traits. `Transform` is a factory that creates middleware instances, while `Service` is what actually processes requests. Understanding these traits is key to building powerful custom middleware.

Let's break down how this works:

### The Transform Trait

`Transform` is a factory that creates middleware instances. It takes a service (your route handler or next middleware) and wraps it with middleware behavior.

```rust
use actix_web::{dev::ServiceRequest, Error};
use actix_service::{Service, Transform};
use std::future::{ready, Ready};
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct CustomLogger;

impl<S, B> Transform<S, ServiceRequest> for CustomLogger
where
    S: Service<ServiceRequest, Response = actix_web::dev::ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    // The response type (passed through from the inner service)
    type Response = actix_web::dev::ServiceResponse<B>;
    
    // The error type (passed through from the inner service)
    type Error = Error;
    
    // Error type if transformation fails (we use () since we can't fail)
    type InitError = ();
    
    // The middleware wrapper type that will wrap the service
    type Transform = CustomLoggerMiddleware<S>;
    
    // The future type for the transformation (Ready means it completes immediately)
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    // new_transform: Factory method that wraps a service with middleware
    // This is called once per worker thread when the app starts
    // It takes the inner service (handler or next middleware) and wraps it
    // ready() creates an immediately-ready future (no async work needed)
    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CustomLoggerMiddleware { service }))
    }
}
```

### The Service Trait

The `Service` trait is what actually processes requests. It's implemented by the middleware wrapper that `Transform` creates.

```rust
pub struct CustomLoggerMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for CustomLoggerMiddleware<S>
where
    S: Service<ServiceRequest, Response = actix_web::dev::ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    // The response type this middleware returns (same as the inner service)
    type Response = actix_web::dev::ServiceResponse<B>;
    
    // The error type this middleware can return (same as the inner service)
    type Error = Error;
    
    // The future type that will resolve to a Response or Error
    // Pin<Box<dyn Future>> allows us to return different future types
    type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    // poll_ready: Called by actix-web to check if this service is ready to process requests
    // This is part of Rust's async runtime - it allows backpressure (pausing when busy)
    // We delegate to the inner service's poll_ready to check if it's ready
    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    // call: This is where the actual middleware logic happens
    // It's called for each request and returns a Future that will resolve to a Response
    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Extract request information before processing
        let path = req.path().to_string();
        let method = req.method().to_string();
        
        // Pre-processing: Log the incoming request
        println!("[CUSTOM] {} {}", method, path);
        
        // Call the next service in the chain (could be another middleware or the handler)
        // This returns a Future that we need to await
        let fut = self.service.call(req);
        
        // Post-processing: We wrap the future to log the response after it completes
        // Box::pin creates a heap-allocated, pinned future (required for trait objects)
        // async move allows us to capture variables and move them into the future
        Box::pin(async move {
            // Await the inner service's response
            let res = fut.await?;
            
            // Post-processing: Log the response status
            println!("[CUSTOM] Response status: {}", res.status());
            
            // Return the response (which may have been modified)
            Ok(res)
        })
    }
}
```

### Complete Custom Logger Example

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse, dev::ServiceRequest, Error};
use actix_service::{Service, Transform};
use std::future::{ready, Ready};
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct CustomLogger;

impl<S, B> Transform<S, ServiceRequest> for CustomLogger
where
    S: Service<ServiceRequest, Response = actix_web::dev::ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = actix_web::dev::ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = CustomLoggerMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CustomLoggerMiddleware { service }))
    }
}

pub struct CustomLoggerMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for CustomLoggerMiddleware<S>
where
    S: Service<ServiceRequest, Response = actix_web::dev::ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = actix_web::dev::ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path().to_string();
        let method = req.method().to_string();
        
        println!("[CUSTOM] {} {}", method, path);
        
        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            println!("[CUSTOM] Response status: {}", res.status());
            Ok(res)
        })
    }
}

async fn hello() -> impl Responder {
    "Hello, World!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(CustomLogger)
            .route("/", web::get().to(hello))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 6: CORS Middleware

CORS (Cross-Origin Resource Sharing) middleware allows your API to be accessed from different origins (domains). This is essential when your frontend runs on a different domain than your backend, or when building public APIs.

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use actix_cors::Cors;

async fn hello() -> impl Responder {
    HttpResponse::Ok().json("Hello, World!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        
        App::new()
            .wrap(cors)
            .route("/", web::get().to(hello))
            .route("/api/data", web::get().to(hello))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 7: Authentication Middleware - Real-World Example

Here's a practical example of protecting API routes with authentication middleware. This middleware checks for an Authorization header and blocks requests that don't have valid credentials, demonstrating how middleware can protect routes.

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse, dev::ServiceRequest, Error};
use actix_service::{Service, Transform};
use std::future::{ready, Ready};
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = actix_web::dev::ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = actix_web::dev::ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService { service }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = actix_web::dev::ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = actix_web::dev::ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Check for Authorization header
        let auth_header = req.headers().get("Authorization");
        
        if let Some(header) = auth_header {
            if let Ok(header_str) = header.to_str() {
                if header_str.starts_with("Bearer ") {
                    let token = &header_str[7..];
                    // In a real app, validate the token against a database or JWT
                    if token == "secret-token" {
                        // Authentication successful, proceed to handler
                        let fut = self.service.call(req);
                        return Box::pin(async move { fut.await });
                    }
                }
            }
        }
        
        // Authentication failed, return error without calling handler
        Box::pin(async move {
            Err(actix_web::error::ErrorUnauthorized("Missing or invalid authorization"))
        })
    }
}

async fn public() -> impl Responder {
    "This is a public endpoint"
}

async fn protected() -> impl Responder {
    "This is a protected endpoint - you are authenticated!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/public", web::get().to(public))
            .service(
                web::scope("/api")
                    .wrap(AuthMiddleware)
                    .route("/protected", web::get().to(protected))
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

**How it works:**
1. The middleware checks for an `Authorization` header
2. If valid, it calls the next service (your handler)
3. If invalid, it returns an error response **without** calling the handler
4. This protects all routes under `/api` scope

## Step 8: Request Timing Middleware

Here's another practical example that measures request processing time. This middleware records how long each request takes to process, which is useful for performance monitoring and debugging slow endpoints.

Here's another practical example that measures request processing time:

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse, dev::ServiceRequest, Error};
use actix_service::{Service, Transform};
use std::future::{ready, Ready};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;

pub struct TimingMiddleware;

impl<S, B> Transform<S, ServiceRequest> for TimingMiddleware
where
    S: Service<ServiceRequest, Response = actix_web::dev::ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = actix_web::dev::ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = TimingMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TimingMiddlewareService { service }))
    }
}

pub struct TimingMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for TimingMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = actix_web::dev::ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = actix_web::dev::ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start = Instant::now();
        let path = req.path().to_string();
        
        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            let duration = start.elapsed();
            println!("Request to {} took {:?}", path, duration);
            Ok(res)
        })
    }
}

async fn hello() -> impl Responder {
    "Hello, World!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(TimingMiddleware)
            .route("/", web::get().to(hello))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 9: Multiple Middleware (Order Matters!)

When you wrap multiple middleware, they execute in **reverse order** (last wrapped = first executed). Understanding this order is crucial because middleware can depend on each other - for example, logging should happen after CORS headers are set.

When you wrap multiple middleware, they execute in **reverse order** (last wrapped = first executed):

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse, middleware::Logger};
use actix_cors::Cors;
use env_logger;

async fn hello() -> impl Responder {
    "Hello, World!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();
        
        App::new()
            // Middleware is applied in REVERSE order!
            // Execution order: CORS → Logger → Handler
            .wrap(Logger::default())
            .wrap(cors)
            .route("/", web::get().to(hello))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

**Execution Flow:**
```
Request → CORS (pre) → Logger (pre) → Handler → Logger (post) → CORS (post) → Response
```

## Step 10: Error Handling Middleware

You can use `ErrorHandlers` middleware to customize error responses. This allows you to add custom headers, modify error messages, or format error responses consistently across your application.

You can use `ErrorHandlers` middleware to customize error responses:

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse, Error, Result, middleware::ErrorHandlers};
use actix_web::http::StatusCode;

async fn hello() -> impl Responder {
    "Hello, World!"
}

async fn error_handler() -> Result<HttpResponse, Error> {
    Err(actix_web::error::ErrorInternalServerError("Something went wrong"))
}

fn add_error_header<B>(mut res: actix_web::dev::ServiceResponse<B>) -> Result<actix_web::dev::ServiceResponse<B>, Error> {
    res.response_mut().headers_mut().insert(
        actix_web::http::header::HeaderName::from_static("x-error"),
        actix_web::http::HeaderValue::from_static("true"),
    );
    Ok(res)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(
                ErrorHandlers::new()
                    .handler(StatusCode::INTERNAL_SERVER_ERROR, add_error_header)
                    .handler(StatusCode::NOT_FOUND, add_error_header)
            )
            .route("/", web::get().to(hello))
            .route("/error", web::get().to(error_handler))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Key Concepts Explained

- **Middleware Order**: Applied in reverse order (last wrapped runs first)
- **Transform Trait**: Factory that creates middleware instances
- **Service Trait**: The actual middleware implementation that processes requests
- **Built-in Middleware**: Logger, CORS, ErrorHandlers, etc.
- **Request/Response Processing**: Middleware can modify both requests and responses
- **Early Returns**: Middleware can return a response without calling the handler

## When to Use Built-in vs Custom Middleware

- **Use Built-in Middleware** for common tasks:
  - `Logger` for request logging
  - `Cors` for cross-origin requests
  - `ErrorHandlers` for custom error responses
  - `DefaultHeaders` for setting default response headers

- **Use Custom Middleware** when you need:
  - Custom authentication logic
  - Request rate limiting
  - Custom request/response transformation
  - Integration with external services

- **Use `wrap_fn`** for simple, one-off middleware that doesn't need to be reusable

## Testing Middleware

You can test middleware behavior:

```bash
# Test public endpoint (no auth required)
curl http://127.0.0.1:8080/public

# Test protected endpoint (auth required)
curl http://127.0.0.1:8080/api/protected
# Returns: 401 Unauthorized

# Test with authentication
curl -H "Authorization: Bearer secret-token" http://127.0.0.1:8080/api/protected
# Returns: 200 OK
```

## Next Steps
In the next chapter, we'll integrate SQLite database to persist data in our application.
