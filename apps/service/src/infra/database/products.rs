use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, DatabaseConnection, DeriveEntityModel, DerivePrimaryKey, DeriveRelation,
    EntityTrait, EnumIter, IntoActiveModel, PrimaryKeyTrait, QueryOrder, Set,
};

use crate::{
    application::products::{
        CreateProductInput, ProductRepository, ProductRepositoryError, UpdateProductInput,
    },
    domain::product::Product,
};

#[derive(Clone)]
pub struct SeaOrmProductRepository {
    db: DatabaseConnection,
}

impl SeaOrmProductRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ProductRepository for SeaOrmProductRepository {
    async fn create(&self, input: CreateProductInput) -> Result<Product, ProductRepositoryError> {
        let product = ActiveModel {
            name: Set(input.name),
            description: Set(input.description),
            price_cents: Set(input.price_cents),
            stock: Set(input.stock),
            ..Default::default()
        }
        .insert(&self.db)
        .await
        .map_err(to_repository_error)?;

        Ok(product.into())
    }

    async fn list(&self) -> Result<Vec<Product>, ProductRepositoryError> {
        let products = Entity::find()
            .order_by_asc(Column::Id)
            .all(&self.db)
            .await
            .map_err(to_repository_error)?;

        Ok(products.into_iter().map(Product::from).collect())
    }

    async fn find(&self, id: i32) -> Result<Product, ProductRepositoryError> {
        let product = Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(to_repository_error)?
            .ok_or(ProductRepositoryError::NotFound)?;

        Ok(product.into())
    }

    async fn update(
        &self,
        id: i32,
        input: UpdateProductInput,
    ) -> Result<Product, ProductRepositoryError> {
        let product = Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(to_repository_error)?
            .ok_or(ProductRepositoryError::NotFound)?;

        if input.name.is_none()
            && input.description.is_none()
            && input.price_cents.is_none()
            && input.stock.is_none()
        {
            return Ok(product.into());
        }

        let mut product = product.into_active_model();

        if let Some(name) = input.name {
            product.name = Set(name);
        }

        if let Some(description) = input.description {
            product.description = Set(description);
        }

        if let Some(price_cents) = input.price_cents {
            product.price_cents = Set(price_cents);
        }

        if let Some(stock) = input.stock {
            product.stock = Set(stock);
        }

        let product = product
            .update(&self.db)
            .await
            .map_err(to_repository_error)?;

        Ok(product.into())
    }

    async fn delete(&self, id: i32) -> Result<(), ProductRepositoryError> {
        let result = Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(to_repository_error)?;

        if result.rows_affected == 0 {
            return Err(ProductRepositoryError::NotFound);
        }

        Ok(())
    }
}

fn to_repository_error(_error: sea_orm::DbErr) -> ProductRepositoryError {
    ProductRepositoryError::Unknown
}

impl From<Model> for Product {
    fn from(product: Model) -> Self {
        Self::new(
            product.id,
            product.name,
            product.description,
            product.price_cents,
            product.stock,
        )
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "products")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub price_cents: i32,
    pub stock: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl sea_orm::ActiveModelBehavior for ActiveModel {}
