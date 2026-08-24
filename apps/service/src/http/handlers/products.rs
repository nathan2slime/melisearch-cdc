use actix_web::{HttpResponse, ResponseError, delete, get, post, put, web};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    application::products::{
        self as product_use_cases, CreateProductInput, ProductError, UpdateProductInput,
    },
    domain::product::Product,
    infra::database::products::SeaOrmProductRepository,
};

#[derive(Deserialize, ToSchema)]
pub struct CreateProductRequest {
    name: String,
    description: Option<String>,
    price_cents: i32,
    stock: i32,
}

impl From<CreateProductRequest> for CreateProductInput {
    fn from(request: CreateProductRequest) -> Self {
        Self {
            name: request.name,
            description: request.description,
            price_cents: request.price_cents,
            stock: request.stock,
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateProductRequest {
    name: String,
    description: Option<String>,
    price_cents: i32,
    stock: i32,
}

impl From<UpdateProductRequest> for UpdateProductInput {
    fn from(request: UpdateProductRequest) -> Self {
        Self {
            name: request.name,
            description: request.description,
            price_cents: request.price_cents,
            stock: request.stock,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct ProductResponse {
    id: i32,
    name: String,
    description: Option<String>,
    price_cents: i32,
    stock: i32,
}

impl From<Product> for ProductResponse {
    fn from(product: Product) -> Self {
        Self {
            id: product.id(),
            name: product.name().to_owned(),
            description: product.description().map(str::to_owned),
            price_cents: product.price_cents(),
            stock: product.stock(),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct ProductErrorResponse {
    message: String,
}

impl ResponseError for ProductError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Self::InvalidName | Self::InvalidPrice | Self::InvalidStock => {
                actix_web::http::StatusCode::BAD_REQUEST
            }
            Self::NotFound => actix_web::http::StatusCode::NOT_FOUND,
            Self::Repository => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ProductErrorResponse {
            message: self.to_string(),
        })
    }
}

#[utoipa::path(
    post,
    path = "/products",
    tag = "Products",
    request_body = CreateProductRequest,
    responses(
        (status = 201, description = "Product created", body = ProductResponse),
        (status = 400, description = "Invalid product payload", body = ProductErrorResponse),
        (status = 500, description = "Product persistence failed", body = ProductErrorResponse)
    )
)]
#[post("/products")]
pub async fn create_product(
    db: web::Data<DatabaseConnection>,
    request: web::Json<CreateProductRequest>,
) -> Result<HttpResponse, ProductError> {
    let repository = product_repository(&db);
    let product =
        product_use_cases::create_product(&repository, request.into_inner().into()).await?;

    Ok(HttpResponse::Created().json(ProductResponse::from(product)))
}

#[utoipa::path(
    get,
    path = "/products",
    tag = "Products",
    responses(
        (status = 200, description = "Products listed", body = Vec<ProductResponse>),
        (status = 500, description = "Product persistence failed", body = ProductErrorResponse)
    )
)]
#[get("/products")]
pub async fn list_products(
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, ProductError> {
    let repository = product_repository(&db);
    let products = product_use_cases::list_products(&repository).await?;
    let products = products
        .into_iter()
        .map(ProductResponse::from)
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(products))
}

#[utoipa::path(
    get,
    path = "/products/{id}",
    tag = "Products",
    params(("id" = i32, Path, description = "Product id")),
    responses(
        (status = 200, description = "Product found", body = ProductResponse),
        (status = 404, description = "Product not found", body = ProductErrorResponse),
        (status = 500, description = "Product persistence failed", body = ProductErrorResponse)
    )
)]
#[get("/products/{id}")]
pub async fn get_product(
    db: web::Data<DatabaseConnection>,
    id: web::Path<i32>,
) -> Result<HttpResponse, ProductError> {
    let repository = product_repository(&db);
    let product = product_use_cases::get_product(&repository, id.into_inner()).await?;

    Ok(HttpResponse::Ok().json(ProductResponse::from(product)))
}

#[utoipa::path(
    put,
    path = "/products/{id}",
    tag = "Products",
    params(("id" = i32, Path, description = "Product id")),
    request_body = UpdateProductRequest,
    responses(
        (status = 200, description = "Product updated", body = ProductResponse),
        (status = 400, description = "Invalid product payload", body = ProductErrorResponse),
        (status = 404, description = "Product not found", body = ProductErrorResponse),
        (status = 500, description = "Product persistence failed", body = ProductErrorResponse)
    )
)]
#[put("/products/{id}")]
pub async fn update_product(
    db: web::Data<DatabaseConnection>,
    id: web::Path<i32>,
    request: web::Json<UpdateProductRequest>,
) -> Result<HttpResponse, ProductError> {
    let repository = product_repository(&db);
    let product = product_use_cases::update_product(
        &repository,
        id.into_inner(),
        request.into_inner().into(),
    )
    .await?;

    Ok(HttpResponse::Ok().json(ProductResponse::from(product)))
}

#[utoipa::path(
    delete,
    path = "/products/{id}",
    tag = "Products",
    params(("id" = i32, Path, description = "Product id")),
    responses(
        (status = 204, description = "Product deleted"),
        (status = 404, description = "Product not found", body = ProductErrorResponse),
        (status = 500, description = "Product persistence failed", body = ProductErrorResponse)
    )
)]
#[delete("/products/{id}")]
pub async fn delete_product(
    db: web::Data<DatabaseConnection>,
    id: web::Path<i32>,
) -> Result<HttpResponse, ProductError> {
    let repository = product_repository(&db);
    product_use_cases::delete_product(&repository, id.into_inner()).await?;

    Ok(HttpResponse::NoContent().finish())
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(create_product)
        .service(list_products)
        .service(get_product)
        .service(update_product)
        .service(delete_product);
}

fn product_repository(db: &web::Data<DatabaseConnection>) -> SeaOrmProductRepository {
    SeaOrmProductRepository::new(db.get_ref().clone())
}
