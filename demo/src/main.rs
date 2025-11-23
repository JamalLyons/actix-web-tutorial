use actix_cors::Cors;
use actix_files as fs;
use actix_service::{Service, Transform};
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::{
    dev::ServiceRequest, middleware::Logger, web, App, Error, HttpRequest, HttpResponse,
    HttpServer, Responder, Result,
};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{FromRow, SqlitePool};
use std::collections::HashMap;
use std::env;
use std::future::{ready, Ready};
use std::pin::Pin;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::Instant;

// Application State
#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    request_count: Arc<Mutex<u64>>,
}

// User Models
#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
struct User {
    id: i64,
    name: String,
    email: String,
    created_at: String,
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

#[derive(Deserialize)]
struct Pagination {
    page: Option<u32>,
    limit: Option<u32>,
}

// GitHub OAuth Models
#[derive(Serialize, Deserialize, Debug, Clone)]
struct GitHubUser {
    id: u64,
    login: String,
    name: Option<String>,
    email: Option<String>,
    avatar_url: String,
}

#[derive(Deserialize)]
struct GitHubTokenResponse {
    access_token: String,
    #[allow(dead_code)] // This is not used in the code but still returned by the GitHub API
    token_type: String,
    #[allow(dead_code)] // This is not used in the code but still returned by the GitHub API
    scope: String,
}

// Handlers

async fn index(_session: Session) -> impl Responder {
    HttpResponse::Ok().body(include_str!("../templates/index.html"))
}

// Get all users
async fn get_users(
    query: web::Query<Pagination>,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(10);
    let offset = (page - 1) * limit;

    let users = sqlx::query_as::<_, User>(
        "SELECT id, name, email, CAST(created_at AS TEXT) as created_at FROM users ORDER BY created_at DESC LIMIT ? OFFSET ?"
    )
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(&state.db)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "data": users,
        "page": page,
        "limit": limit,
    })))
}

// Get user by ID
async fn get_user(path: web::Path<i64>, state: web::Data<AppState>) -> Result<HttpResponse> {
    let user_id = path.into_inner();

    let user = sqlx::query_as::<_, User>(
        "SELECT id, name, email, CAST(created_at AS TEXT) as created_at FROM users WHERE id = ?",
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    match user {
        Some(user) => Ok(HttpResponse::Ok().json(user)),
        None => Ok(HttpResponse::NotFound().json("User not found")),
    }
}

// Create user
async fn create_user(
    user: web::Json<CreateUser>,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let result = sqlx::query("INSERT INTO users (name, email) VALUES (?, ?)")
        .bind(&user.name)
        .bind(&user.email)
        .execute(&state.db)
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint") {
                actix_web::error::ErrorConflict("Email already exists")
            } else {
                actix_web::error::ErrorInternalServerError(e)
            }
        })?;

    let created_user = sqlx::query_as::<_, User>(
        "SELECT id, name, email, CAST(created_at AS TEXT) as created_at FROM users WHERE id = ?",
    )
    .bind(result.last_insert_rowid())
    .fetch_one(&state.db)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Created().json(created_user))
}

// Helper function to get current authenticated user
fn get_current_user(session: &Session) -> Option<GitHubUser> {
    session.get("github_user").unwrap_or(None)
}

// Get current authenticated user
async fn get_current_user_api(session: Session) -> Result<HttpResponse> {
    match get_current_user(&session) {
        Some(user) => Ok(HttpResponse::Ok().json(user)),
        None => Ok(HttpResponse::Unauthorized().json("Not authenticated")),
    }
}

// GitHub OAuth Login
async fn github_login() -> Result<HttpResponse> {
    let client_id = env::var("GITHUB_CLIENT_ID").expect("GITHUB_CLIENT_ID must be set");

    let redirect_uri = "http://localhost:8080/auth/github/callback";
    let scope = "user:email";
    let state = "random_state_string"; // In production, use a random, secure state

    let auth_url = format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope={}&state={}",
        client_id, redirect_uri, scope, state
    );

    Ok(HttpResponse::Found()
        .append_header(("Location", auth_url))
        .finish())
}

// GitHub OAuth Callback
async fn github_callback(
    session: Session,
    query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse> {
    let code = query
        .get("code")
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Missing code"))?;

    // Exchange code for access token
    let client_id = env::var("GITHUB_CLIENT_ID").expect("GITHUB_CLIENT_ID must be set");
    let client_secret = env::var("GITHUB_CLIENT_SECRET").expect("GITHUB_CLIENT_SECRET must be set");

    let token_url = "https://github.com/login/oauth/access_token";
    let client = reqwest::Client::new();

    let token_response = client
        .post(token_url)
        .header("Accept", "application/json")
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", code),
        ])
        .send()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    let token_data: GitHubTokenResponse = token_response
        .json()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // Get user info from GitHub
    let user_response = client
        .get("https://api.github.com/user")
        .bearer_auth(&token_data.access_token)
        .header("User-Agent", "actix-web-tutorial")
        .send()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    let github_user: GitHubUser = user_response
        .json()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // Store user in session
    session
        .insert("github_user", &github_user)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Found()
        .append_header(("Location", "/"))
        .finish())
}

// Logout
async fn logout(session: Session) -> impl Responder {
    session.purge();
    HttpResponse::Found()
        .append_header(("Location", "/"))
        .finish()
}

// Update user - requires authentication
async fn update_user(
    path: web::Path<i64>,
    user: web::Json<UpdateUser>,
    state: web::Data<AppState>,
    session: Session,
) -> Result<HttpResponse> {
    // Check authentication
    if get_current_user(&session).is_none() {
        return Ok(HttpResponse::Unauthorized().json("Authentication required"));
    }

    let user_id = path.into_inner();

    // Check if user exists and get current values
    let current =
        sqlx::query_as::<_, (String, String)>("SELECT name, email FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    let (current_name, current_email) = match current {
        Some((name, email)) => (name, email),
        None => return Ok(HttpResponse::NotFound().json("User not found")),
    };

    // Use provided values or keep existing ones
    let name = user.name.as_ref().unwrap_or(&current_name);
    let email = user.email.as_ref().unwrap_or(&current_email);

    // Simple update query
    sqlx::query("UPDATE users SET name = ?, email = ? WHERE id = ?")
        .bind(name)
        .bind(email)
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    let updated_user = sqlx::query_as::<_, User>(
        "SELECT id, name, email, CAST(created_at AS TEXT) as created_at FROM users WHERE id = ?",
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().json(updated_user))
}

// Delete user - requires authentication
async fn delete_user(
    path: web::Path<i64>,
    state: web::Data<AppState>,
    session: Session,
) -> Result<HttpResponse> {
    // Check authentication
    if get_current_user(&session).is_none() {
        return Ok(HttpResponse::Unauthorized().json("Authentication required"));
    }

    let user_id = path.into_inner();

    let result = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    if result.rows_affected() == 0 {
        Ok(HttpResponse::NotFound().json("User not found"))
    } else {
        Ok(HttpResponse::Ok().json("User deleted"))
    }
}

// Request counter
async fn get_counter(state: web::Data<AppState>) -> impl Responder {
    let mut count = state.request_count.lock().unwrap();
    *count += 1;
    HttpResponse::Ok().json(serde_json::json!({
        "request_count": *count,
    }))
}

// Request info
async fn request_info(req: HttpRequest) -> impl Responder {
    let method = req.method();
    let path = req.path();
    let headers = req.headers();
    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("Unknown");

    HttpResponse::Ok().json(serde_json::json!({
        "method": method.to_string(),
        "path": path,
        "user_agent": user_agent,
    }))
}

// Form submission
async fn submit_form(form: web::Form<CreateUser>) -> impl Responder {
    HttpResponse::Ok().body(format!(
        r#"<div class="alert alert-success">
            <p>Welcome, {}! Your account has been created.</p>
        </div>"#,
        form.name
    ))
}

// Public endpoint
async fn public() -> impl Responder {
    "This is a public endpoint"
}

// Protected endpoint
async fn protected() -> impl Responder {
    "This is a protected endpoint - you are authenticated!"
}

// Middleware Examples

// Custom Timing Middleware
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
            log::info!("Request to {} took {:?}", path, duration);
            Ok(res)
        })
    }
}

// Authentication Middleware
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
            Err(actix_web::error::ErrorUnauthorized(
                "Missing or invalid authorization",
            ))
        })
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Database setup
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:demo.db".to_string());

    let connect_options = SqliteConnectOptions::from_str(&database_url)
        .expect("Invalid database URL")
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options)
        .await
        .expect("Failed to connect to database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Session secret (must be at least 64 bytes/512 bits)
    let secret_key = match env::var("SESSION_SECRET") {
        Ok(secret) if secret.as_bytes().len() >= 64 => Key::from(secret.as_bytes()),
        _ => {
            log::warn!("SESSION_SECRET not set or too short, generating a new key");
            Key::generate()
        }
    };

    // Application state
    let app_state = web::Data::new(AppState {
        db: pool,
        request_count: Arc::new(Mutex::new(0)),
    });

    log::info!("Starting server on http://127.0.0.1:8080");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(app_state.clone())
            .app_data(web::JsonConfig::default().limit(4096))
            .app_data(web::FormConfig::default().limit(1024))
            // Middleware
            // Order: TimingMiddleware → Logger → CORS → SessionMiddleware
            // Execution: SessionMiddleware → CORS → Logger → TimingMiddleware → Handler
            .wrap(TimingMiddleware)
            .wrap(Logger::default())
            .wrap(cors)
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_secure(false)
                    .cookie_http_only(true)
                    .build(),
            )
            // Static routes
            .route("/", web::get().to(index))
            // Authentication routes
            .route("/auth/github/login", web::get().to(github_login))
            .route("/auth/github/callback", web::get().to(github_callback))
            .route("/auth/logout", web::get().to(logout))
            // API routes
            .route("/api/users", web::get().to(get_users))
            .route("/api/users", web::post().to(create_user))
            .route("/api/users/{id}", web::get().to(get_user))
            .route("/api/users/{id}", web::put().to(update_user))
            .route("/api/users/{id}", web::delete().to(delete_user))
            .route("/api/me", web::get().to(get_current_user_api))
            // Demo routes
            .route("/counter", web::get().to(get_counter))
            .route("/request-info", web::get().to(request_info))
            // HTMX route
            .route("/htmx/submit", web::post().to(submit_form))
            // Middleware demo routes
            .route("/public", web::get().to(public))
            .service(
                web::scope("/api")
                    .wrap(AuthMiddleware)
                    .route("/protected", web::get().to(protected)),
            )
            // Static files
            .service(fs::Files::new("/static", "./static"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
