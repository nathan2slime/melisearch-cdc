mod application;
mod config;
mod domain;
mod http;
mod infra;

use config::Config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Config::from_env();
    init_logging();

    http::server::run(config).await
}

fn init_logging() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
}
