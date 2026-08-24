mod errors;
mod inputs;
mod outputs;
mod ports;
mod use_cases;

pub use errors::{ProductError, ProductRepositoryError, ProductSearchIndexError};
pub use inputs::{CreateProductInput, SearchProductsInput, UpdateProductInput};
pub use outputs::{ProductSearchIndexOutput, SearchProductsOutput};
pub use ports::{ProductRepository, ProductSearchIndex};
pub use use_cases::{create_product, delete_product, get_product, search_products, update_product};
