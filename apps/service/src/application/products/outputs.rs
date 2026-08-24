use crate::domain::product::Product;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchProductsOutput {
    pub items: Vec<Product>,
    pub page: u64,
    pub per_page: u64,
    pub total_items: u64,
    pub total_pages: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductSearchIndexOutput {
    pub ids: Vec<i32>,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}
