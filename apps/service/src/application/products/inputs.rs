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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchProductsInput {
    pub query: String,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}
