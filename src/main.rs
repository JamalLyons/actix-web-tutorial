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
