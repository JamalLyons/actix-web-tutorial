use actix_web::{
    dev::{Service, ServiceRequest, Transform},
    web, App, Error, HttpServer, Responder,
};
use std::future::{ready, Ready};
use std::pin::Pin;
use std::task::{Context, Poll};

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

async fn public() -> impl Responder {
    "This is a public endpoint"
}

async fn protected() -> impl Responder {
    "This is a protected endpoint - you are authenticated!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().route("/public", web::get().to(public)).service(
            web::scope("/api")
                .wrap(AuthMiddleware)
                .route("/protected", web::get().to(protected)),
        )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
