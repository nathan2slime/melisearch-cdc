mod application;
mod config;
mod domain;
mod http;
mod infra;

use config::Config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    http::server::run(Config::from_env()).await
}
