# Chapter 4: Serving Static Content

## Overview
In this chapter, we'll learn how to serve static files (CSS, JavaScript, images, and other assets) using actix-web. This is essential for building web applications that need to serve client-side resources.

## Step 1: Update Cargo.toml

We need to add `actix-files` to serve static files like CSS, JavaScript, and images from the filesystem.

```toml
[dependencies]
actix-files = "0.6.8"
```

## Step 2: Create Directory Structure

Organize your static files into a dedicated directory. This keeps your project clean and makes it easy to manage assets.

```bash
mkdir -p static/css
mkdir -p templates
```

## Step 3: Basic Static File Serving

The `actix-files` crate provides a service to serve static files from a directory. You mount a directory to a URL path, and actix-web will serve files from that directory.

```rust
use actix_web::{web, App, HttpServer};
use actix_files as fs;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(
                fs::Files::new("/static", "./static")
                    .show_files_listing()
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

**How it works:**
- `fs::Files::new("/static", "./static")` creates a service that serves files from the `./static` directory
- The first parameter (`"/static"`) is the URL path prefix
- The second parameter (`"./static"`) is the filesystem path
- `.show_files_listing()` enables directory listing (useful for development, disable in production)

## Step 4: Serving HTML Files

You can serve HTML files as static content, or embed them directly in your binary using `include_str!` for better performance.

**Option 1: Serve HTML as static files**

```rust
use actix_web::{web, App, HttpServer};
use actix_files as fs;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(fs::Files::new("/", "./static").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

**Option 2: Embed HTML in binary**

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use actix_files as fs;

async fn index() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../templates/index.html"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .service(fs::Files::new("/static", "./static"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

The `include_str!` macro embeds the file content at compile time, making it part of your binary. This is faster but requires recompiling when the file changes.

## Step 5: Complete Example with Multiple Static Directories

You can serve multiple directories and configure them differently:

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use actix_files as fs;

async fn index() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../templates/index.html"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            // Serve CSS, JS, and other assets
            .service(fs::Files::new("/static", "./static"))
            // Serve images from a separate directory
            .service(fs::Files::new("/images", "./images"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 6: Static File Configuration Options

The `Files` service provides several configuration options:

```rust
use actix_web::{web, App, HttpServer};
use actix_files as fs;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(
                fs::Files::new("/static", "./static")
                    .show_files_listing()        // Show directory listing
                    .prefer_utf8(true)           // Prefer UTF-8 encoding
                    .default_handler(            // Custom 404 handler
                        web::route().to(|| async {
                            HttpResponse::NotFound().body("File not found")
                        })
                    )
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 7: Serving Files with Custom Headers

You can add custom headers to static file responses:

```rust
use actix_web::{web, App, HttpServer, HttpRequest, Result};
use actix_files::{Files, NamedFile};
use std::path::PathBuf;

async fn serve_file(req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    let file = NamedFile::open(path)?;
    Ok(file
        .set_content_type(mime::TEXT_HTML)
        .customize()
        .insert_header(("X-Custom-Header", "value"))
    )
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/file/{filename:.*}", web::get().to(serve_file))
            .service(fs::Files::new("/static", "./static"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Key Concepts Explained

- **`actix-files`**: Crate for serving static files
- **`Files::new()`**: Creates a service to serve files from a directory
- **URL Path vs Filesystem Path**: The URL path (first parameter) doesn't have to match the filesystem path (second parameter)
- **`include_str!`**: Macro to embed file contents at compile time
- **Directory Listing**: Useful for development, but should be disabled in production
- **Multiple Directories**: You can serve multiple directories with different URL prefixes

## Best Practices

- **Disable directory listing in production**: Remove `.show_files_listing()` for security
- **Use `include_str!` for small, frequently accessed files**: Reduces I/O operations
- **Organize static files**: Keep CSS, JS, and images in separate subdirectories
- **Set appropriate content types**: actix-files handles this automatically, but you can customize if needed
- **Consider CDN for production**: For better performance, serve static assets from a CDN

## Common Use Cases

- **CSS Stylesheets**: Serve from `/static/css/`
- **JavaScript Files**: Serve from `/static/js/`
- **Images**: Serve from `/static/images/` or `/images/`
- **Fonts**: Serve from `/static/fonts/`
- **Favicons**: Serve from root or `/static/`

## Next Steps
In the next chapter, we'll implement GitHub OAuth authentication to secure our application.
