# Chapter 2: Routing

## Overview
Routing is fundamental to web applications. In this chapter, we'll learn how to define routes, handle different HTTP methods, use route patterns, organize routes with scopes, and understand route precedence in actix-web.

## Step 1: Basic Route Registration

The simplest way to register a route is using `.route()` with a path pattern and HTTP method. Actix-web matches incoming requests to these routes based on the URL path and HTTP method.

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse};

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello, World!")
}

async fn about() -> impl Responder {
    HttpResponse::Ok().body("About page")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/about", web::get().to(about))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 2: HTTP Methods

The same URL path can handle different HTTP methods. Each method typically represents a different action: GET for reading, POST for creating, PUT for updating, PATCH for partial updates, and DELETE for removing resources.

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse};

async fn get_users() -> impl Responder {
    HttpResponse::Ok().body("List of users")
}

async fn create_user() -> impl Responder {
    HttpResponse::Created().body("User created")
}

async fn update_user() -> impl Responder {
    HttpResponse::Ok().body("User updated")
}

async fn delete_user() -> impl Responder {
    HttpResponse::Ok().body("User deleted")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/users", web::get().to(get_users))
            .route("/users", web::post().to(create_user))
            .route("/users", web::put().to(update_user))
            .route("/users", web::delete().to(delete_user))
            .route("/users", web::patch().to(update_user))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 3: Route Patterns and Wildcards

Actix-web supports various route patterns:
- `{name}` - Captures a single segment
- `{name:.*}` - Captures the rest of the path (greedy match)
- Static paths match exactly

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse};

async fn static_route() -> impl Responder {
    HttpResponse::Ok().body("Static route")
}

async fn catch_all() -> impl Responder {
    HttpResponse::Ok().body("Catch-all route")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/static", web::get().to(static_route))
            .route("/files/{path:.*}", web::get().to(catch_all))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 4: Route Scoping with web::scope

Route scoping allows you to group related routes under a common path prefix. This is useful for organizing your API endpoints and applying middleware to groups of routes.

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse};

async fn get_users() -> impl Responder {
    HttpResponse::Ok().body("List users")
}

async fn get_user() -> impl Responder {
    HttpResponse::Ok().body("Get user")
}

async fn get_posts() -> impl Responder {
    HttpResponse::Ok().body("List posts")
}

async fn get_post() -> impl Responder {
    HttpResponse::Ok().body("Get post")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(
                web::scope("/api")
                    .route("/users", web::get().to(get_users))
                    .route("/users/{id}", web::get().to(get_user))
            )
            .service(
                web::scope("/v1")
                    .route("/posts", web::get().to(get_posts))
                    .route("/posts/{id}", web::get().to(get_post))
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

The routes above will be accessible at:
- `/api/users`
- `/api/users/{id}`
- `/v1/posts`
- `/v1/posts/{id}`

## Step 5: Nested Scopes

You can nest scopes to create hierarchical route structures. This is useful for organizing complex APIs.

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse};

async fn get_user() -> impl Responder {
    HttpResponse::Ok().body("Get user")
}

async fn get_user_posts() -> impl Responder {
    HttpResponse::Ok().body("Get user posts")
}

async fn get_user_post() -> impl Responder {
    HttpResponse::Ok().body("Get specific user post")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(
                web::scope("/api")
                    .service(
                        web::scope("/users/{user_id}")
                            .route("", web::get().to(get_user))
                            .route("/posts", web::get().to(get_user_posts))
                            .route("/posts/{post_id}", web::get().to(get_user_post))
                    )
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

The routes above will be accessible at:
- `/api/users/{user_id}`
- `/api/users/{user_id}/posts`
- `/api/users/{user_id}/posts/{post_id}`

## Step 6: Resource Routing

The `web::resource()` method provides a convenient way to register multiple HTTP methods for a single path pattern. This is cleaner than registering each method separately.

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse};

async fn get_users() -> impl Responder {
    HttpResponse::Ok().body("List users")
}

async fn create_user() -> impl Responder {
    HttpResponse::Created().body("Create user")
}

async fn get_user() -> impl Responder {
    HttpResponse::Ok().body("Get user")
}

async fn update_user() -> impl Responder {
    HttpResponse::Ok().body("Update user")
}

async fn delete_user() -> impl Responder {
    HttpResponse::Ok().body("Delete user")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(
                web::resource("/users")
                    .route(web::get().to(get_users))
                    .route(web::post().to(create_user))
            )
            .service(
                web::resource("/users/{id}")
                    .route(web::get().to(get_user))
                    .route(web::put().to(update_user))
                    .route(web::delete().to(delete_user))
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 7: Route Precedence and Ordering

Routes are matched in the order they are registered. More specific routes should be registered before more general ones. If a catch-all route is registered first, it will match all requests before more specific routes can be checked.

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse};

async fn specific() -> impl Responder {
    HttpResponse::Ok().body("Specific route")
}

async fn catch_all() -> impl Responder {
    HttpResponse::Ok().body("Catch-all route")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            // Register specific routes first
            .route("/users/settings", web::get().to(specific))
            // Then register more general routes
            .route("/users/{id}", web::get().to(catch_all))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

**Important**: If you register `/users/{id}` before `/users/settings`, the `{id}` route will match `/users/settings` with `id = "settings"`, and the specific route will never be reached.

## Step 8: Default Route and 404 Handling

You can register a default route that matches any path that hasn't been matched by previous routes. This is useful for handling 404 errors or serving a default page.

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse};

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Home page")
}

async fn not_found() -> impl Responder {
    HttpResponse::NotFound().body("Page not found")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .default_service(web::route().to(not_found))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 9: Route Guards

Route guards allow you to conditionally match routes based on request properties. You can use guards to match routes based on headers, query parameters, or other request attributes.

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse, guard};

async fn api_v1() -> impl Responder {
    HttpResponse::Ok().body("API v1")
}

async fn api_v2() -> impl Responder {
    HttpResponse::Ok().body("API v2")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(
                web::scope("/api")
                    .route(
                        "/users",
                        web::get()
                            .guard(guard::Header("X-API-Version", "v1"))
                            .to(api_v1)
                    )
                    .route(
                        "/users",
                        web::get()
                            .guard(guard::Header("X-API-Version", "v2"))
                            .to(api_v2)
                    )
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 10: Complete Example - RESTful API Structure

This example demonstrates a complete RESTful API structure using all the routing concepts we've covered.

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse};

// User handlers
async fn list_users() -> impl Responder {
    HttpResponse::Ok().body("List all users")
}

async fn get_user() -> impl Responder {
    HttpResponse::Ok().body("Get user by ID")
}

async fn create_user() -> impl Responder {
    HttpResponse::Created().body("Create new user")
}

async fn update_user() -> impl Responder {
    HttpResponse::Ok().body("Update user")
}

async fn delete_user() -> impl Responder {
    HttpResponse::Ok().body("Delete user")
}

// Post handlers
async fn list_posts() -> impl Responder {
    HttpResponse::Ok().body("List all posts")
}

async fn get_post() -> impl Responder {
    HttpResponse::Ok().body("Get post by ID")
}

// Health check
async fn health() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

// 404 handler
async fn not_found() -> impl Responder {
    HttpResponse::NotFound().json("Not found")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            // Health check route
            .route("/health", web::get().to(health))
            // API routes organized by resource
            .service(
                web::scope("/api")
                    .service(
                        web::resource("/users")
                            .route(web::get().to(list_users))
                            .route(web::post().to(create_user))
                    )
                    .service(
                        web::resource("/users/{id}")
                            .route(web::get().to(get_user))
                            .route(web::put().to(update_user))
                            .route(web::delete().to(delete_user))
                    )
                    .service(
                        web::resource("/posts")
                            .route(web::get().to(list_posts))
                    )
                    .service(
                        web::resource("/posts/{id}")
                            .route(web::get().to(get_post))
                    )
            )
            // Default route for unmatched paths
            .default_service(web::route().to(not_found))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Key Concepts Explained

- **Route Registration**: Use `.route()` for individual routes or `.service()` with `web::resource()` for multiple methods
- **HTTP Methods**: `web::get()`, `web::post()`, `web::put()`, `web::patch()`, `web::delete()`, etc.
- **Route Scoping**: Use `web::scope()` to group routes under a common prefix
- **Nested Scopes**: Scopes can be nested to create hierarchical route structures
- **Resource Routing**: Use `web::resource()` to register multiple HTTP methods for one path
- **Route Precedence**: Routes are matched in registration order; register specific routes before general ones
- **Default Routes**: Use `.default_service()` to handle unmatched requests
- **Route Guards**: Use guards to conditionally match routes based on request properties

## Route Matching Rules

1. Routes are matched in the order they are registered
2. More specific routes should be registered before more general ones
3. Path parameters (`{name}`) match any single segment
4. Greedy patterns (`{name:.*}`) match the rest of the path
5. Static paths match exactly
6. Scopes add a prefix to all routes within them

## Testing Your Routes

You can test these routes using:
- Browser (for GET requests)
- `curl`:
  ```bash
  # Test different HTTP methods
  curl http://127.0.0.1:8080/users
  curl -X POST http://127.0.0.1:8080/users
  curl -X PUT http://127.0.0.1:8080/users/1
  curl -X DELETE http://127.0.0.1:8080/users/1
  
  # Test scoped routes
  curl http://127.0.0.1:8080/api/users
  curl http://127.0.0.1:8080/api/users/1
  
  # Test path parameters
  curl http://127.0.0.1:8080/users/123
  curl http://127.0.0.1:8080/users/123/posts/456
  
  # Test route guards
  curl -H "X-API-Version: v1" http://127.0.0.1:8080/api/users
  curl -H "X-API-Version: v2" http://127.0.0.1:8080/api/users
  ```

## Next Steps
In the next chapter, we'll dive deep into extractors and learn how to extract and use path parameters, query strings, request bodies, and other data from HTTP requests.
