use actix_files as fs;
use actix_web::{web, App, HttpResponse, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().service(
            fs::Files::new("/static", "./static")
                .show_files_listing() // Show directory listing
                .prefer_utf8(true) // Prefer UTF-8 encoding
                .default_handler(
                    // Custom 404 handler
                    web::route().to(|| async { HttpResponse::NotFound().body("File not found") }),
                ),
        )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
