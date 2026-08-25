pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub meilisearch_url: String,
    pub meilisearch_api_key: Option<String>,
    pub meilisearch_products_index: String,
}

impl Config {
    pub fn from_env() -> Self {
        let _ = dotenvy::dotenv();

        let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_owned());
        let port = std::env::var("PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(8080);
        let database_url = std::env::var("DATABASE_URL").unwrap();
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
            host,
            port,
            database_url,
            meilisearch_url,
            meilisearch_api_key,
            meilisearch_products_index,
        }
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
