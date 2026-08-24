#[derive(Debug, Eq, PartialEq)]
pub enum ProductError {
    InvalidName,
    InvalidPrice,
    InvalidStock,
    InvalidSearchPage,
    InvalidSearchPageSize,
    NotFound,
    Repository,
    SearchIndex,
}

impl std::fmt::Display for ProductError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidName => formatter.write_str("product name is required"),
            Self::InvalidPrice => formatter.write_str("product price must be zero or greater"),
            Self::InvalidStock => formatter.write_str("product stock must be zero or greater"),
            Self::InvalidSearchPage => {
                formatter.write_str("product search page must be 1 or greater")
            }
            Self::InvalidSearchPageSize => {
                formatter.write_str("product search per_page must be between 1 and 100")
            }
            Self::NotFound => formatter.write_str("product not found"),
            Self::Repository => formatter.write_str("product persistence failed"),
            Self::SearchIndex => formatter.write_str("product search failed"),
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

#[derive(Debug, Eq, PartialEq)]
pub enum ProductSearchIndexError {
    Infrastructure(String),
}

impl From<ProductSearchIndexError> for ProductError {
    fn from(_error: ProductSearchIndexError) -> Self {
        Self::SearchIndex
    }
}
