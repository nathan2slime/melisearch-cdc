use async_trait::async_trait;

use crate::domain::product::Product;

use super::{
    errors::{ProductRepositoryError, ProductSearchIndexError},
    inputs::{CreateProductInput, UpdateProductInput},
    outputs::ProductSearchIndexOutput,
};

#[async_trait]
pub trait ProductRepository {
    async fn create(&self, input: CreateProductInput) -> Result<Product, ProductRepositoryError>;
    async fn find(&self, id: i32) -> Result<Product, ProductRepositoryError>;
    async fn find_many(&self, ids: &[i32]) -> Result<Vec<Product>, ProductRepositoryError>;
    async fn update(
        &self,
        id: i32,
        input: UpdateProductInput,
    ) -> Result<Product, ProductRepositoryError>;
    async fn delete(&self, id: i32) -> Result<(), ProductRepositoryError>;
}

#[async_trait]
pub trait ProductSearchIndex {
    async fn search(
        &self,
        query: &str,
        page: u64,
        per_page: u64,
    ) -> Result<ProductSearchIndexOutput, ProductSearchIndexError>;
    async fn count_documents(&self) -> Result<u64, ProductSearchIndexError>;
}
