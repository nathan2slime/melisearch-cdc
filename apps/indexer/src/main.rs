mod application;
mod config;
mod domain;
mod infra;

use application::products::run_product_indexer;
use config::Config;
use infra::{kafka::KafkaProductEventSource, meilisearch::MeilisearchProductIndex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = Config::from_env();
    let product_index = MeilisearchProductIndex::new(
        config.meilisearch_url.clone(),
        config.meilisearch_api_key.clone(),
        config.meilisearch_products_index.clone(),
    );

    let event_source = KafkaProductEventSource::new(
        &config.kafka_bootstrap_servers,
        &config.kafka_group_id,
        config.kafka_products_topic.clone(),
    )?;

    println!(
        "indexer consuming Kafka topic '{}' from '{}'",
        config.kafka_products_topic, config.kafka_bootstrap_servers
    );

    run_product_indexer(&event_source, &product_index).await?;

    Ok(())
}
