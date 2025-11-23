# Chapter 1: Basic Setup

## Overview

## Step 1: Create a New Rust Project

We'll start by creating a new Rust project using Cargo, which will set up the basic project structure and configuration files.

```bash
cargo new actix-tutorial
cd actix-tutorial
```

## Step 2: Add Dependencies to Cargo.toml

Add actix-web to your project dependencies. This crate provides all the functionality we need to build web applications in Rust.

```toml
[dependencies]
actix-web = "4.12.0"
```

## Step 3: Basic Hello World Server

This is our first actix-web server. We create an HTTP server that listens on port 8080 and responds with "Hello, World!" when you visit the root path.

Replace the contents of `src/main.rs`:

```rust
use actix_web::{web, App, HttpServer, Responder};

async fn hello() -> impl Responder {
    "Hello, World!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(hello))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 4: Run the Server

Compile and start the server using Cargo. Once running, you can access it through your web browser.

```bash
cargo run
```

Visit `http://127.0.0.1:8080` in your browser to see "Hello, World!"

## Key Concepts Explained

- **`HttpServer::new()`**: Creates a new HTTP server instance
- **`App::new()`**: Creates a new application instance
- **`.route()`**: Defines a route with a path and handler
- **`web::get()`**: Specifies the HTTP method (GET in this case)
- **`#[actix_web::main]`**: Macro that sets up the async runtime
- **`.bind()`**: Binds the server to an address and port
- **`.run().await`**: Starts the server and waits for it to run

## Next Steps
In the next chapter, we'll explore routing in more detail, including different HTTP methods, path parameters, and query parameters.

