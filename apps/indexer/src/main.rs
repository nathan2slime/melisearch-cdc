mod application;
mod config;
mod domain;
mod infra;

use std::time::Duration;

use application::products::{retry_until_ok, run_product_indexer};
use config::Config;
use infra::{kafka::KafkaProductEventSource, meilisearch::MeilisearchProductIndex};

const INDEXER_RETRY_DELAY: Duration = Duration::from_secs(1);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = Config::from_env();
    let product_index = MeilisearchProductIndex::new(
        config.meilisearch_url.clone(),
        config.meilisearch_api_key.clone(),
        config.meilisearch_products_index.clone(),
    );

    let event_source = retry_until_ok("create Kafka event source", INDEXER_RETRY_DELAY, || async {
        KafkaProductEventSource::new(
            &config.kafka_bootstrap_servers,
            &config.kafka_group_id,
            config.kafka_products_topic.clone(),
        )
    })
    .await;

    println!(
        "indexer consuming Kafka topic '{}' from '{}' in batches of up to {} messages",
        config.kafka_products_topic,
        config.kafka_bootstrap_servers,
        config.kafka_products_batch_size
    );

    run_product_indexer(
        &event_source,
        &product_index,
        config.kafka_products_batch_size,
        Duration::from_millis(config.kafka_products_batch_max_wait_ms),
        INDEXER_RETRY_DELAY,
    )
    .await?;

    Ok(())
}
