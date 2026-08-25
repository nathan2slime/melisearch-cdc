use actix_cors::Cors;
use actix_web::{App, HttpServer, web};

use crate::{
    config::Config,
    http::{docs, handlers},
    infra::{database::connect, search::products::MeilisearchProductSearchIndex},
};

pub async fn run(config: Config) -> std::io::Result<()> {
    let db = web::Data::new(connect::connect(&config.database_url).await?);
    let product_search_index = web::Data::new(MeilisearchProductSearchIndex::new(
        config.meilisearch_url.clone(),
        config.meilisearch_api_key.clone(),
        config.meilisearch_products_index.clone(),
    ));
    let bind_address = config.bind_address();

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(db.clone())
            .app_data(product_search_index.clone())
            .service(
                web::scope("/api")
                    .configure(handlers::health::configure)
                    .configure(handlers::products::configure),
            )
            .service(docs::swagger_ui())
    })
    .bind(bind_address)?
    .run()
    .await
}
