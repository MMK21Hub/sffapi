use std::net::Ipv4Addr;

use actix_web::{App, HttpServer, Responder, get, web};
use serde::Serialize;
use tracing::{Level, event};

#[derive(Serialize)]
struct HealthCheckResponse {
    status: String,
}

#[get("/health")]
async fn health_check() -> impl Responder {
    web::Json(HealthCheckResponse {
        status: "OK".to_string(),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    let port = 4640;
    let bind_address = Ipv4Addr::LOCALHOST;

    event!(Level::INFO, "starting server");
    HttpServer::new(|| {
        App::new() //
            .service(health_check)
    })
    .bind((bind_address, port))?
    .run()
    .await
}
