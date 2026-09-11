use std::{
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
};

use actix_web::{App, HttpResponse, HttpServer, Responder, get, web};
use argh::FromArgs;
use serde::Serialize;
use tracing::{Level, event};

use crate::{database::get_mini_pc, structs::MiniPCStats};

mod data;
mod database;
mod structs;

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
struct MiniPCResponse {
    id: i64,
    title: String,
    cpu: Option<String>,
    hostname: Option<String>,
    deployed: bool,
    stats: Option<MiniPCStats>,
}

#[derive(Serialize)]
struct ApiError {
    error: &'static str,
    message: String,
}

#[get("/device/{id}")]
async fn mini_pc(
    id: web::Path<i64>,
    pool: web::Data<database::Pool>,
    cache: web::Data<data::StatsCache>,
) -> impl Responder {
    let id = id.into_inner();

    let connection = match pool.get() {
        Ok(connection) => connection,
        Err(err) => {
            event!(Level::ERROR, "failed to get database connection: {err:#}");
            return HttpResponse::InternalServerError().json(ApiError {
                error: "internal_error",
                message: "internal server error".to_string(),
            });
        }
    };

    let physical = match get_mini_pc(&connection, id) {
        Ok(Some(physical)) => physical,
        Ok(None) => {
            return HttpResponse::NotFound().json(ApiError {
                error: "not_found",
                message: format!("no mini PC with id {id}"),
            });
        }
        Err(err) => {
            event!(Level::ERROR, "failed to load mini pc {id}: {err:#}");
            return HttpResponse::InternalServerError().json(ApiError {
                error: "database_error",
                message: "failed to read mini PC from the database".to_string(),
            });
        }
    };

    let stats = match data::get_stats(&cache, id).await {
        Ok(stats) => Some(stats),
        Err(err) => {
            event!(
                Level::WARN,
                "failed to fetch stats for mini pc {id}: {err:#}"
            );
            None
        }
    };

    HttpResponse::Ok().json(MiniPCResponse {
        id: physical.id,
        title: physical.title,
        cpu: physical.cpu,
        hostname: physical.hostname,
        deployed: physical.deployed,
        stats,
    })
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
            .app_data(web::Data::new(data::StatsCache::default()))
            .service(health_check)
            .service(mini_pc)
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
