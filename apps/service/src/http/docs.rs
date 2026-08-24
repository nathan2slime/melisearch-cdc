use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::http::handlers::health::{__path_health, HealthIndicatorResponse, HealthResponse};

#[derive(OpenApi)]
#[openapi(
    paths(
        health,
    ),
    components(schemas(HealthResponse, HealthIndicatorResponse)),
    tags(
        (name = "health")
    )
)]
struct ApiDoc;

pub fn swagger_ui() -> SwaggerUi {
    SwaggerUi::new("/docs/{_:.*}").url("/api-docs/openapi.json", ApiDoc::openapi())
}
