use crate::domain::product::Product;

use super::{
    errors::ProductError,
    inputs::{CreateProductInput, SearchProductsInput, UpdateProductInput},
    outputs::SearchProductsOutput,
    ports::{ProductRepository, ProductSearchIndex},
};

const DEFAULT_SEARCH_PAGE: u64 = 1;
const DEFAULT_SEARCH_PER_PAGE: u64 = 20;
const MAX_SEARCH_PER_PAGE: u64 = 100;

pub async fn create_product(
    repository: &impl ProductRepository,
    input: CreateProductInput,
) -> Result<Product, ProductError> {
    validate_product_fields(&input.name, input.price_cents, input.stock)?;

    repository.create(input).await.map_err(ProductError::from)
}

pub async fn get_product(
    repository: &impl ProductRepository,
    id: i32,
) -> Result<Product, ProductError> {
    repository.find(id).await.map_err(ProductError::from)
}

pub async fn search_products(
    repository: &impl ProductRepository,
    search_index: &impl ProductSearchIndex,
    input: SearchProductsInput,
) -> Result<SearchProductsOutput, ProductError> {
    let query = input.query.trim();

    let page = input.page.unwrap_or(DEFAULT_SEARCH_PAGE);
    if page == 0 {
        return Err(ProductError::InvalidSearchPage);
    }

    let per_page = input.per_page.unwrap_or(DEFAULT_SEARCH_PER_PAGE);
    if per_page == 0 || per_page > MAX_SEARCH_PER_PAGE {
        return Err(ProductError::InvalidSearchPageSize);
    }

    let search_result = search_index
        .search(query, page, per_page)
        .await
        .map_err(ProductError::from)?;
    let items = repository
        .find_many(&search_result.ids)
        .await
        .map_err(ProductError::from)?;
    let total_items = search_index
        .count_documents()
        .await
        .map_err(ProductError::from)?;

    Ok(SearchProductsOutput {
        items,
        page: search_result.page,
        per_page: search_result.per_page,
        total_items,
        total_pages: search_result.total_pages,
    })
}

pub async fn update_product(
    repository: &impl ProductRepository,
    id: i32,
    input: UpdateProductInput,
) -> Result<Product, ProductError> {
    validate_update_product_fields(&input)?;

    repository
        .update(id, input)
        .await
        .map_err(ProductError::from)
}

pub async fn delete_product(
    repository: &impl ProductRepository,
    id: i32,
) -> Result<(), ProductError> {
    repository.delete(id).await.map_err(ProductError::from)
}

fn validate_product_fields(name: &str, price_cents: i32, stock: i32) -> Result<(), ProductError> {
    if name.trim().is_empty() {
        return Err(ProductError::InvalidName);
    }

    if price_cents < 0 {
        return Err(ProductError::InvalidPrice);
    }

    if stock < 0 {
        return Err(ProductError::InvalidStock);
    }

    Ok(())
}

fn validate_update_product_fields(input: &UpdateProductInput) -> Result<(), ProductError> {
    if input
        .name
        .as_deref()
        .is_some_and(|name| name.trim().is_empty())
    {
        return Err(ProductError::InvalidName);
    }

    if input.price_cents.is_some_and(|price_cents| price_cents < 0) {
        return Err(ProductError::InvalidPrice);
    }

    if input.stock.is_some_and(|stock| stock < 0) {
        return Err(ProductError::InvalidStock);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;

    use super::*;
    use crate::application::products::{
        ProductRepositoryError, ProductSearchIndexError, ProductSearchIndexOutput,
    };

    #[derive(Default)]
    struct InMemoryProductRepository {
        products: Vec<Product>,
    }

    impl InMemoryProductRepository {
        fn new(products: Vec<Product>) -> Self {
            Self { products }
        }
    }

    #[async_trait]
    impl ProductRepository for InMemoryProductRepository {
        async fn create(
            &self,
            input: CreateProductInput,
        ) -> Result<Product, ProductRepositoryError> {
            Ok(Product::new(
                1,
                input.name,
                input.description,
                input.price_cents,
                input.stock,
            ))
        }

        async fn find(&self, id: i32) -> Result<Product, ProductRepositoryError> {
            self.products
                .iter()
                .find(|product| product.id() == id)
                .cloned()
                .ok_or(ProductRepositoryError::NotFound)
        }

        async fn find_many(&self, ids: &[i32]) -> Result<Vec<Product>, ProductRepositoryError> {
            Ok(ids
                .iter()
                .filter_map(|id| {
                    self.products
                        .iter()
                        .find(|product| product.id() == *id)
                        .cloned()
                })
                .collect())
        }

        async fn update(
            &self,
            _id: i32,
            input: UpdateProductInput,
        ) -> Result<Product, ProductRepositoryError> {
            Ok(Product::new(
                1,
                input.name.unwrap_or_else(|| "Keyboard".to_owned()),
                input.description.unwrap_or(Some("Mechanical".to_owned())),
                input.price_cents.unwrap_or(100),
                input.stock.unwrap_or(1),
            ))
        }

        async fn delete(&self, _id: i32) -> Result<(), ProductRepositoryError> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct InMemoryProductSearchIndex {
        ids: Vec<i32>,
        expected_query: String,
        expected_page: u64,
        expected_per_page: u64,
        total_items: u64,
        total_pages: u64,
    }

    #[async_trait]
    impl ProductSearchIndex for InMemoryProductSearchIndex {
        async fn search(
            &self,
            query: &str,
            page: u64,
            per_page: u64,
        ) -> Result<ProductSearchIndexOutput, ProductSearchIndexError> {
            assert_eq!(query, self.expected_query);
            assert_eq!(page, self.expected_page);
            assert_eq!(per_page, self.expected_per_page);

            Ok(ProductSearchIndexOutput {
                ids: self.ids.clone(),
                page,
                per_page,
                total_pages: self.total_pages,
            })
        }

        async fn count_documents(&self) -> Result<u64, ProductSearchIndexError> {
            Ok(self.total_items)
        }
    }

    #[tokio::test]
    async fn create_product_rejects_empty_name() {
        let repository = InMemoryProductRepository::default();

        let result = create_product(
            &repository,
            CreateProductInput {
                name: " ".to_owned(),
                description: None,
                price_cents: 100,
                stock: 1,
            },
        )
        .await;

        assert_eq!(result, Err(ProductError::InvalidName));
    }

    #[tokio::test]
    async fn update_product_rejects_negative_price() {
        let repository = InMemoryProductRepository::default();

        let result = update_product(
            &repository,
            1,
            UpdateProductInput {
                name: None,
                description: None,
                price_cents: Some(-1),
                stock: None,
            },
        )
        .await;

        assert_eq!(result, Err(ProductError::InvalidPrice));
    }

    #[tokio::test]
    async fn update_product_accepts_partial_name_update() {
        let repository = InMemoryProductRepository::default();

        let product = update_product(
            &repository,
            1,
            UpdateProductInput {
                name: Some("Mouse".to_owned()),
                description: None,
                price_cents: None,
                stock: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(product.name(), "Mouse");
        assert_eq!(product.description(), Some("Mechanical"));
        assert_eq!(product.price_cents(), 100);
        assert_eq!(product.stock(), 1);
    }

    #[tokio::test]
    async fn update_product_accepts_description_clear() {
        let repository = InMemoryProductRepository::default();

        let product = update_product(
            &repository,
            1,
            UpdateProductInput {
                name: None,
                description: Some(None),
                price_cents: None,
                stock: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(product.name(), "Keyboard");
        assert_eq!(product.description(), None);
        assert_eq!(product.price_cents(), 100);
        assert_eq!(product.stock(), 1);
    }

    #[tokio::test]
    async fn search_products_rejects_zero_page() {
        let repository = InMemoryProductRepository::default();
        let search_index = InMemoryProductSearchIndex::default();

        let result = search_products(
            &repository,
            &search_index,
            SearchProductsInput {
                query: "keyboard".to_owned(),
                page: Some(0),
                per_page: None,
            },
        )
        .await;

        assert_eq!(result, Err(ProductError::InvalidSearchPage));
    }

    #[tokio::test]
    async fn search_products_rejects_oversized_page_size() {
        let repository = InMemoryProductRepository::default();
        let search_index = InMemoryProductSearchIndex::default();

        let result = search_products(
            &repository,
            &search_index,
            SearchProductsInput {
                query: "keyboard".to_owned(),
                page: None,
                per_page: Some(101),
            },
        )
        .await;

        assert_eq!(result, Err(ProductError::InvalidSearchPageSize));
    }

    #[tokio::test]
    async fn search_products_accepts_empty_query_for_meilisearch_defaults() {
        let repository =
            InMemoryProductRepository::new(vec![Product::new(1, "Mouse", None, 5999, 5)]);
        let search_index = InMemoryProductSearchIndex {
            ids: vec![1],
            expected_query: "".to_owned(),
            expected_page: 1,
            expected_per_page: 20,
            total_items: 1,
            total_pages: 1,
        };

        let result = search_products(
            &repository,
            &search_index,
            SearchProductsInput {
                query: " ".to_owned(),
                page: None,
                per_page: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(result.items.len(), 1);
        assert_eq!(result.total_items, 1);
    }

    #[tokio::test]
    async fn search_products_returns_paginated_pg_products_in_search_order() {
        let repository = InMemoryProductRepository::new(vec![
            Product::new(1, "Mouse", None, 5999, 5),
            Product::new(2, "Keyboard", Some("Mechanical".to_owned()), 12999, 7),
            Product::new(3, "Monitor", None, 99999, 2),
        ]);
        let search_index = InMemoryProductSearchIndex {
            ids: vec![2, 1],
            expected_query: "keyboard".to_owned(),
            expected_page: 2,
            expected_per_page: 2,
            total_items: 4,
            total_pages: 2,
        };

        let result = search_products(
            &repository,
            &search_index,
            SearchProductsInput {
                query: " keyboard ".to_owned(),
                page: Some(2),
                per_page: Some(2),
            },
        )
        .await
        .unwrap();

        let ids = result
            .items
            .into_iter()
            .map(|product| product.id())
            .collect::<Vec<_>>();

        assert_eq!(ids, vec![2, 1]);
        assert_eq!(result.page, 2);
        assert_eq!(result.per_page, 2);
        assert_eq!(result.total_items, 4);
        assert_eq!(result.total_pages, 2);
    }
}
