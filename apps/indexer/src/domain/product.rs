#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductDocument {
    pub id: i32,
    pub name: String,
}

impl ProductDocument {
    pub fn new(id: i32, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProductEvent {
    Upsert(ProductDocument),
    Delete(i32),
    Ignore,
}
