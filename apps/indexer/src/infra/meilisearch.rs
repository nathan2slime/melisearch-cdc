use async_trait::async_trait;
use reqwest::{RequestBuilder, StatusCode};
use serde::Serialize;

use crate::{
    application::products::{ProductIndex, ProductIndexError},
    domain::product::ProductDocument,
};

#[derive(Clone)]
pub struct MeilisearchProductIndex {
    client: reqwest::Client,
    url: String,
    api_key: Option<String>,
    products_index: String,
}

impl MeilisearchProductIndex {
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
impl ProductIndex for MeilisearchProductIndex {
    async fn ensure_ready(&self) -> Result<(), ProductIndexError> {
        let response = self
            .authorize(self.client.post(format!("{}/indexes", self.url)))
            .json(&CreateIndexRequest {
                uid: &self.products_index,
                primary_key: "id",
            })
            .send()
            .await
            .map_err(to_index_error)?;

        let status = response.status();
        let body = response.text().await.map_err(to_index_error)?;

        if status.is_success() || body.contains("index_already_exists") {
            return Ok(());
        }

        Err(meilisearch_response_error(status, body))
    }

    async fn upsert_products(&self, products: &[ProductDocument]) -> Result<(), ProductIndexError> {
        if products.is_empty() {
            return Ok(());
        }

        let products = products
            .iter()
            .map(ProductDocumentRequest::from)
            .collect::<Vec<_>>();
        let response = self
            .authorize(self.client.post(format!(
                "{}/indexes/{}/documents",
                self.url, self.products_index
            )))
            .json(&products)
            .send()
            .await
            .map_err(to_index_error)?;

        accept_meilisearch_response(response).await
    }

    async fn delete_products(&self, ids: &[i32]) -> Result<(), ProductIndexError> {
        if ids.is_empty() {
            return Ok(());
        }

        let response = self
            .authorize(self.client.post(format!(
                "{}/indexes/{}/documents/delete-batch",
                self.url, self.products_index
            )))
            .json(ids)
            .send()
            .await
            .map_err(to_index_error)?;

        accept_meilisearch_response(response).await
    }
}

async fn accept_meilisearch_response(response: reqwest::Response) -> Result<(), ProductIndexError> {
    let status = response.status();

    if status.is_success() {
        return Ok(());
    }

    let body = response.text().await.map_err(to_index_error)?;

    Err(meilisearch_response_error(status, body))
}

fn to_index_error(error: reqwest::Error) -> ProductIndexError {
    ProductIndexError::Infrastructure(error.to_string())
}

fn meilisearch_response_error(status: StatusCode, body: String) -> ProductIndexError {
    ProductIndexError::Infrastructure(format!("Meilisearch request failed with {status}: {body}"))
}

#[derive(Serialize)]
struct CreateIndexRequest<'a> {
    uid: &'a str,
    #[serde(rename = "primaryKey")]
    primary_key: &'a str,
}

#[derive(Serialize)]
struct ProductDocumentRequest<'a> {
    id: i32,
    name: &'a str,
}

impl<'a> From<&'a ProductDocument> for ProductDocumentRequest<'a> {
    fn from(product: &'a ProductDocument) -> Self {
        Self {
            id: product.id,
            name: &product.name,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn serializes_only_product_id_and_name_for_meilisearch() {
        let product = ProductDocument::new(1, "Keyboard");
        let request = ProductDocumentRequest::from(&product);
        let value = serde_json::to_value(request).unwrap();

        assert_eq!(
            value,
            json!({
                "id": 1,
                "name": "Keyboard"
            })
        );
    }
}
