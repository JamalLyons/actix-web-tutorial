use actix_web::{web, App, HttpResponse, HttpServer, Responder};
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
        email: user
            .email
            .clone()
            .unwrap_or_else(|| "unknown@example.com".to_string()),
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
