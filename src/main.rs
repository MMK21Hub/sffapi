use std::{
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
};

use actix_web::{App, HttpServer, Responder, get, web};
use argh::FromArgs;
use serde::Serialize;
use tracing::{Level, event};

mod database;

#[derive(FromArgs)]
/// Unified Mini PC API
struct Sffapi {
    /// path to the database file (default: data/sffapi.db)
    #[argh(option, default = "PathBuf::from(\"data/sffapi.db\")")]
    database: PathBuf,
    /// port to listen on (default: 4640)
    #[argh(option, default = "4640")]
    port: u16,
    /// address to bind to (default: 0.0.0.0)
    #[argh(option, default = "IpAddr::V4(Ipv4Addr::UNSPECIFIED)")]
    bind: IpAddr,
}

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

async fn run_main() -> anyhow::Result<()> {
    let args: Sffapi = argh::from_env();

    let pool = database::get_db_pool(&args.database)?;

    event!(Level::INFO, "starting server");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(health_check)
    })
    .bind((args.bind, args.port))?
    .run()
    .await?;

    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    match run_main().await {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            event!(Level::ERROR, "{}", e);
            std::process::exit(1);
        }
    }
}
