use async_trait::async_trait;

use crate::domain::product::Product;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateProductInput {
    pub name: String,
    pub description: Option<String>,
    pub price_cents: i32,
    pub stock: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdateProductInput {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub price_cents: Option<i32>,
    pub stock: Option<i32>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ProductError {
    InvalidName,
    InvalidPrice,
    InvalidStock,
    NotFound,
    Repository,
}

impl std::fmt::Display for ProductError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidName => formatter.write_str("product name is required"),
            Self::InvalidPrice => formatter.write_str("product price must be zero or greater"),
            Self::InvalidStock => formatter.write_str("product stock must be zero or greater"),
            Self::NotFound => formatter.write_str("product not found"),
            Self::Repository => formatter.write_str("product persistence failed"),
        }
    }
}

impl std::error::Error for ProductError {}

#[derive(Debug, Eq, PartialEq)]
pub enum ProductRepositoryError {
    NotFound,
    Unknown,
}

impl From<ProductRepositoryError> for ProductError {
    fn from(error: ProductRepositoryError) -> Self {
        match error {
            ProductRepositoryError::NotFound => Self::NotFound,
            ProductRepositoryError::Unknown => Self::Repository,
        }
    }
}

#[async_trait]
pub trait ProductRepository {
    async fn create(&self, input: CreateProductInput) -> Result<Product, ProductRepositoryError>;
    async fn list(&self) -> Result<Vec<Product>, ProductRepositoryError>;
    async fn find(&self, id: i32) -> Result<Product, ProductRepositoryError>;
    async fn update(
        &self,
        id: i32,
        input: UpdateProductInput,
    ) -> Result<Product, ProductRepositoryError>;
    async fn delete(&self, id: i32) -> Result<(), ProductRepositoryError>;
}

pub async fn create_product(
    repository: &impl ProductRepository,
    input: CreateProductInput,
) -> Result<Product, ProductError> {
    validate_product_fields(&input.name, input.price_cents, input.stock)?;

    repository.create(input).await.map_err(ProductError::from)
}

pub async fn list_products(
    repository: &impl ProductRepository,
) -> Result<Vec<Product>, ProductError> {
    repository.list().await.map_err(ProductError::from)
}

pub async fn get_product(
    repository: &impl ProductRepository,
    id: i32,
) -> Result<Product, ProductError> {
    repository.find(id).await.map_err(ProductError::from)
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
    use super::*;

    struct InMemoryProductRepository;

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

        async fn list(&self) -> Result<Vec<Product>, ProductRepositoryError> {
            Ok(vec![])
        }

        async fn find(&self, _id: i32) -> Result<Product, ProductRepositoryError> {
            Err(ProductRepositoryError::NotFound)
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

    #[tokio::test]
    async fn create_product_rejects_empty_name() {
        let repository = InMemoryProductRepository;

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
        let repository = InMemoryProductRepository;

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
        let repository = InMemoryProductRepository;

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
        let repository = InMemoryProductRepository;

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
}
