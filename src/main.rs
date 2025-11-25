use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware, SessionExt};
use actix_web::cookie::Key;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web, App, Error, HttpResponse, HttpServer, Responder, Result,
};
use futures_util::future::{self, LocalBoxFuture};
use serde::{Deserialize, Serialize};
use std::env;
use std::rc::Rc;

pub struct Auth;

impl<S, B> Transform<S, ServiceRequest> for Auth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future = future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        future::ready(Ok(AuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct AuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let session = req.get_session();
        let is_logged_in = session.get::<User>("user").unwrap_or(None).is_some();
        let srv = self.service.clone();

        Box::pin(async move {
            if !is_logged_in {
                return Err(actix_web::error::ErrorUnauthorized("Not authenticated"));
            }

            srv.call(req).await
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct User {
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

#[derive(Deserialize, Debug)]
struct GitHubUser {
    id: u64,
    login: String,
    name: Option<String>,
    email: Option<String>,
    avatar_url: String,
}

async fn index(session: Session) -> impl Responder {
    let user: Option<User> = session.get("user").unwrap_or(None);

    if let Some(user) = user {
        HttpResponse::Ok().body(format!(
            r#"
            <html>
                <body>
                    <h1>Welcome, {}!</h1>
                    <img src="{}" alt="Avatar" width="50" height="50">
                    <p>Email: {}</p>
                    <a href="/logout">Logout</a>
                </body>
            </html>
            "#,
            user.login,
            user.avatar_url,
            user.email.as_ref().unwrap_or(&"Not provided".to_string())
        ))
    } else {
        HttpResponse::Ok().body(
            r#"
            <html>
                <body>
                    <h1>Welcome!</h1>
                    <a href="/auth/github/login">Login with GitHub</a>
                </body>
            </html>
            "#,
        )
    }
}

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

async fn github_callback(
    session: Session,
    query: web::Query<std::collections::HashMap<String, String>>,
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

    dbg!(&github_user);

    // Store user in session
    let user = User {
        id: github_user.id,
        login: github_user.login,
        name: github_user.name,
        email: github_user.email,
        avatar_url: github_user.avatar_url,
    };

    session
        .insert("user", &user)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Found()
        .append_header(("Location", "/"))
        .finish())
}

async fn logout(session: Session) -> impl Responder {
    session.purge();
    HttpResponse::Found()
        .append_header(("Location", "/"))
        .finish()
}

async fn protected_route(session: Session) -> impl Responder {
    let user: User = session
        .get("user")
        .unwrap()
        .expect("User should be authenticated");

    HttpResponse::Ok().json(format!("Protected content for {}", user.login))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let secret_key = Key::generate();

    HttpServer::new(move || {
        App::new()
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_secure(false) // Set to true in production with HTTPS
                    .cookie_http_only(true)
                    .build(),
            )
            .route("/", web::get().to(index))
            .route("/auth/github/login", web::get().to(github_login))
            .route("/auth/github/callback", web::get().to(github_callback))
            .route("/logout", web::get().to(logout))
            .service(
                web::scope("/api")
                    .wrap(Auth)
                    .route("/protected", web::get().to(protected_route)),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
