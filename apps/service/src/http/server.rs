use actix_web::{App, HttpServer, web};

use crate::{
    config::Config,
    http::{docs, handlers},
    infra::database::connect,
};

pub async fn run(config: Config) -> std::io::Result<()> {
    let db = web::Data::new(connect::connect(&config.database_url).await?);
    let bind_address = config.bind_address();

    HttpServer::new(move || {
        App::new()
            .app_data(db.clone())
            .configure(handlers::health::configure)
            .configure(handlers::products::configure)
            .service(docs::swagger_ui())
    })
    .bind(bind_address)?
    .run()
    .await
}
