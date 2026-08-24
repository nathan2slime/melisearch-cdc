use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::http::handlers::{
    health::{__path_health, HealthIndicatorResponse, HealthResponse},
    products::{
        __path_create_product, __path_delete_product, __path_get_product, __path_search_products,
        __path_update_product, CreateProductRequest, ProductErrorResponse, ProductResponse,
        SearchProductsQuery, SearchProductsResponse, UpdateProductRequest,
    },
};

#[derive(OpenApi)]
#[openapi(
    paths(
        health,
        create_product,
        search_products,
        get_product,
        update_product,
        delete_product,
    ),
    components(schemas(
        HealthResponse,
        HealthIndicatorResponse,
        CreateProductRequest,
        UpdateProductRequest,
        SearchProductsQuery,
        SearchProductsResponse,
        ProductResponse,
        ProductErrorResponse
    )),
    servers((url = "/api")),
    tags(
        (name = "Health"),
        (name = "Products")
    )
)]
struct ApiDoc;

pub fn swagger_ui() -> SwaggerUi {
    SwaggerUi::new("/docs/{_:.*}").url("/api-docs/openapi.json", ApiDoc::openapi())
}
