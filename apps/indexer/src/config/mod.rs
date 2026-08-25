pub struct Config {
    pub kafka_bootstrap_servers: String,
    pub kafka_products_topic: String,
    pub kafka_group_id: String,
    pub kafka_products_batch_size: usize,
    pub kafka_products_batch_max_wait_ms: u64,
    pub meilisearch_url: String,
    pub meilisearch_api_key: Option<String>,
    pub meilisearch_products_index: String,
}

impl Config {
    pub fn from_env() -> Self {
        let _ = dotenvy::dotenv();

        let kafka_bootstrap_servers = std::env::var("KAFKA_BOOTSTRAP_SERVERS")
            .unwrap_or_else(|_| "localhost:9092".to_owned());
        let kafka_products_topic = std::env::var("KAFKA_PRODUCTS_TOPIC")
            .unwrap_or_else(|_| "melisearch.public.products".to_owned());
        let kafka_group_id =
            std::env::var("KAFKA_GROUP_ID").unwrap_or_else(|_| "melisearch-indexer".to_owned());
        let kafka_products_batch_size = std::env::var("KAFKA_PRODUCTS_BATCH_SIZE")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(5_000);
        let kafka_products_batch_max_wait_ms = std::env::var("KAFKA_PRODUCTS_BATCH_MAX_WAIT_MS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(500);
        let meilisearch_url = std::env::var("MEILISEARCH_URL")
            .unwrap_or_else(|_| "http://localhost:7700".to_owned())
            .trim_end_matches('/')
            .to_owned();
        let meilisearch_api_key = std::env::var("MEILISEARCH_API_KEY")
            .ok()
            .filter(|api_key| !api_key.is_empty());
        let meilisearch_products_index =
            std::env::var("MEILISEARCH_PRODUCTS_INDEX").unwrap_or_else(|_| "products".to_owned());

        Self {
            kafka_bootstrap_servers,
            kafka_products_topic,
            kafka_group_id,
            kafka_products_batch_size,
            kafka_products_batch_max_wait_ms,
            meilisearch_url,
            meilisearch_api_key,
            meilisearch_products_index,
        }
    }
}
