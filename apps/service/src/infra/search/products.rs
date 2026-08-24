use async_trait::async_trait;
use reqwest::{RequestBuilder, StatusCode};
use serde::{Deserialize, Serialize};

use crate::application::products::{
    ProductSearchIndex, ProductSearchIndexError, ProductSearchIndexOutput,
};

#[derive(Clone)]
pub struct MeilisearchProductSearchIndex {
    client: reqwest::Client,
    url: String,
    api_key: Option<String>,
    products_index: String,
}

impl MeilisearchProductSearchIndex {
    pub fn new(url: String, api_key: Option<String>, products_index: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            url,
            api_key,
            products_index,
        }
    }

    fn authorize(&self, request: RequestBuilder) -> RequestBuilder {
        match &self.api_key {
            Some(api_key) => request.bearer_auth(api_key),
            None => request,
        }
    }
}

#[async_trait]
impl ProductSearchIndex for MeilisearchProductSearchIndex {
    async fn search(
        &self,
        query: &str,
        page: u64,
        per_page: u64,
    ) -> Result<ProductSearchIndexOutput, ProductSearchIndexError> {
        let response = self
            .authorize(self.client.post(format!(
                "{}/indexes/{}/search",
                self.url, self.products_index
            )))
            .json(&SearchProductsRequest {
                q: query,
                page,
                hits_per_page: per_page,
            })
            .send()
            .await
            .map_err(to_search_index_error)?;
        let status = response.status();

        if !status.is_success() {
            let body = response.text().await.map_err(to_search_index_error)?;

            return Err(meilisearch_response_error(status, body));
        }

        let response = response
            .json::<SearchProductsResponse>()
            .await
            .map_err(to_search_index_error)?;

        Ok(ProductSearchIndexOutput {
            ids: response.hits.into_iter().map(|hit| hit.id).collect(),
            page: response.page,
            per_page: response.hits_per_page,
            total_pages: response.total_pages,
        })
    }

    async fn count_documents(&self) -> Result<u64, ProductSearchIndexError> {
        let response = self
            .authorize(self.client.get(format!(
                "{}/indexes/{}/stats",
                self.url, self.products_index
            )))
            .send()
            .await
            .map_err(to_search_index_error)?;
        let status = response.status();

        if !status.is_success() {
            let body = response.text().await.map_err(to_search_index_error)?;

            return Err(meilisearch_response_error(status, body));
        }

        let response = response
            .json::<IndexStatsResponse>()
            .await
            .map_err(to_search_index_error)?;

        Ok(response.number_of_documents)
    }
}

fn to_search_index_error(error: reqwest::Error) -> ProductSearchIndexError {
    ProductSearchIndexError::Infrastructure(error.to_string())
}

fn meilisearch_response_error(status: StatusCode, body: String) -> ProductSearchIndexError {
    ProductSearchIndexError::Infrastructure(format!(
        "Meilisearch search request failed with {status}: {body}"
    ))
}

#[derive(Serialize)]
struct SearchProductsRequest<'a> {
    q: &'a str,
    page: u64,
    #[serde(rename = "hitsPerPage")]
    hits_per_page: u64,
}

#[derive(Deserialize)]
struct SearchProductsResponse {
    hits: Vec<SearchProductHit>,
    page: u64,
    #[serde(rename = "hitsPerPage")]
    hits_per_page: u64,
    #[serde(rename = "totalPages")]
    total_pages: u64,
}

#[derive(Deserialize)]
struct SearchProductHit {
    id: i32,
}

#[derive(Deserialize)]
struct IndexStatsResponse {
    #[serde(rename = "numberOfDocuments")]
    number_of_documents: u64,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn serializes_meilisearch_search_request() {
        let request = SearchProductsRequest {
            q: "keyboard",
            page: 1,
            hits_per_page: 20,
        };
        let value = serde_json::to_value(request).unwrap();

        assert_eq!(
            value,
            json!({
                "q": "keyboard",
                "page": 1,
                "hitsPerPage": 20
            })
        );
    }
}
