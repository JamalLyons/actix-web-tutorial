# Chapter 5: Authentication with GitHub OAuth

## Overview
In this chapter, we'll implement GitHub OAuth authentication to allow users to log in with their GitHub accounts. We'll use sessions to maintain user state.

## Step 1: Update Cargo.toml

We need several crates for authentication: `actix-session` for managing user sessions, `reqwest` for making HTTP requests to GitHub's API, and `actix-web-httpauth` for HTTP authentication.

```toml
[dependencies]
actix-session = { version = "0.8", features = ["cookie"] }
actix-web-httpauth = "0.8"
reqwest = { version = "0.12", features = ["json"] }
```

## Step 2: GitHub OAuth Setup

Before we can authenticate users, we need to register our application with GitHub. This gives us a Client ID and Client Secret that identify our app to GitHub's OAuth service.

First, create a GitHub OAuth App:
1. Go to GitHub Settings → Developer settings → OAuth Apps
2. Click "New OAuth App"
3. Set Authorization callback URL: `http://localhost:8080/auth/github/callback`
4. Save your Client ID and Client Secret

## Step 3: Environment Variables

Store sensitive credentials like API keys and secrets in environment variables, not in your code. The `.env` file loads these variables when your application starts.

Create a `.env` file (add to `.gitignore`):

```env
GITHUB_CLIENT_ID=your_client_id_here
GITHUB_CLIENT_SECRET=your_client_secret_here
SESSION_SECRET=your_random_secret_key_here_min_32_chars
```

Add `dotenv` to Cargo.toml:

```toml
[dependencies]
# ... existing dependencies ...
dotenv = "0.15"
```

## Step 4: Basic Session Setup

Sessions allow you to store user data across multiple requests. The `SessionMiddleware` manages session cookies, and you can store and retrieve data using the `Session` extractor.

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse, HttpRequest};
use actix_session::{Session, SessionMiddleware, storage::CookieSessionStore};
use actix_web::cookie::Key;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: u64,
    login: String,
    name: Option<String>,
    email: Option<String>,
    avatar_url: String,
}

async fn index(session: Session) -> impl Responder {
    let user: Option<User> = session.get("user").unwrap_or(None);
    
    if let Some(user) = user {
        HttpResponse::Ok().json(format!("Welcome, {}!", user.login))
    } else {
        HttpResponse::Ok().body(
            r#"
            <html>
                <body>
                    <h1>Welcome!</h1>
                    <a href="/auth/github/login">Login with GitHub</a>
                </body>
            </html>
            "#
        )
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    
    let secret_key = Key::generate(); // In production, use a fixed key from env
    
    HttpServer::new(move || {
        App::new()
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_secure(false) // Set to true in production with HTTPS
                    .build()
            )
            .route("/", web::get().to(index))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 5: GitHub OAuth Flow Implementation

The OAuth flow works in three steps: redirect the user to GitHub, receive the authorization code, and exchange it for an access token. Then we use the token to fetch the user's GitHub profile and store it in the session.

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse, HttpRequest, Result};
use actix_session::{Session, SessionMiddleware, storage::CookieSessionStore};
use actix_web::cookie::Key;
use serde::{Deserialize, Serialize};
use std::env;

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

#[derive(Deserialize)]
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
            "#
        )
    }
}

async fn github_login() -> Result<HttpResponse> {
    let client_id = env::var("GITHUB_CLIENT_ID")
        .expect("GITHUB_CLIENT_ID must be set");
    
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
    let code = query.get("code")
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Missing code"))?;
    
    // Exchange code for access token
    let client_id = env::var("GITHUB_CLIENT_ID")
        .expect("GITHUB_CLIENT_ID must be set");
    let client_secret = env::var("GITHUB_CLIENT_SECRET")
        .expect("GITHUB_CLIENT_SECRET must be set");
    
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
    let user = User {
        id: github_user.id,
        login: github_user.login,
        name: github_user.name,
        email: github_user.email,
        avatar_url: github_user.avatar_url,
    };
    
    session.insert("user", &user)
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    
    let secret_key = Key::from(
        env::var("SESSION_SECRET")
            .expect("SESSION_SECRET must be set")
            .as_bytes()
    );
    
    HttpServer::new(move || {
        App::new()
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_secure(false) // Set to true in production with HTTPS
                    .cookie_http_only(true)
                    .build()
            )
            .route("/", web::get().to(index))
            .route("/auth/github/login", web::get().to(github_login))
            .route("/auth/github/callback", web::get().to(github_callback))
            .route("/logout", web::get().to(logout))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 6: Protected Routes with Middleware

You can protect routes by checking if a user is authenticated before allowing access. Middleware functions can intercept requests and return errors if authentication is missing.

```rust
use actix_web::{web, App, HttpServer, Responder, HttpResponse, HttpRequest, Error};
use actix_session::{Session, SessionMiddleware, storage::CookieSessionStore};
use actix_web::cookie::Key;
use actix_web::dev::ServiceRequest;
use actix_web_lab::middleware::from_fn;
use serde::{Deserialize, Serialize};
use std::env;

// ... User struct and other types ...

async fn require_auth(
    req: ServiceRequest,
    session: Session,
) -> Result<ServiceRequest, Error> {
    let user: Option<User> = session.get("user").unwrap_or(None);
    
    if user.is_none() {
        return Err(actix_web::error::ErrorUnauthorized("Not authenticated"));
    }
    
    Ok(req)
}

async fn protected_route(session: Session) -> impl Responder {
    let user: User = session.get("user")
        .unwrap()
        .expect("User should be authenticated");
    
    HttpResponse::Ok().json(format!("Protected content for {}", user.login))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    
    let secret_key = Key::from(
        env::var("SESSION_SECRET")
            .expect("SESSION_SECRET must be set")
            .as_bytes()
    );
    
    HttpServer::new(move || {
        App::new()
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_secure(false)
                    .cookie_http_only(true)
                    .build()
            )
            .route("/", web::get().to(index))
            .route("/auth/github/login", web::get().to(github_login))
            .route("/auth/github/callback", web::get().to(github_callback))
            .route("/logout", web::get().to(logout))
            .service(
                web::scope("/api")
                    .wrap(from_fn(require_auth))
                    .route("/protected", web::get().to(protected_route))
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Step 7: Helper Function to Get Current User

Create helper functions to extract user information from sessions. This reduces code duplication and makes your handlers cleaner and easier to read.

```rust
fn get_current_user(session: &Session) -> Option<User> {
    session.get("user").unwrap_or(None)
}

async fn profile(session: Session) -> impl Responder {
    match get_current_user(&session) {
        Some(user) => HttpResponse::Ok().json(user),
        None => HttpResponse::Unauthorized().json("Not authenticated"),
    }
}
```

## Key Concepts Explained

- **OAuth 2.0 Flow**: Authorization code flow with GitHub
- **Sessions**: Store user data in encrypted cookies
- **Session Middleware**: Automatically handles session management
- **Protected Routes**: Use middleware to require authentication
- **Environment Variables**: Store secrets securely

## Security Best Practices

1. Always use HTTPS in production
2. Set `cookie_secure(true)` in production
3. Use a strong, random `SESSION_SECRET`
4. Validate the OAuth state parameter
5. Store tokens securely (consider using Redis for sessions in production)

## Next Steps
In the next chapter, we'll explore middleware in more detail, including logging, CORS, and custom middleware.
