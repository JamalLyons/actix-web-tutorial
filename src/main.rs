use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
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

async fn get_users(query: web::Query<Pagination>, data: web::Data<AppState>) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(10);
    let users = data.users.lock().unwrap();

    HttpResponse::Ok().json(format!(
        "Page: {}, Limit: {}, Users: {}",
        page,
        limit,
        users.len()
    ))
}

async fn create_user(
    path: web::Path<u32>,
    body: web::Json<CreateUser>,
    req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    let account_id = path.into_inner();
    let user_agent = req
        .headers()
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
