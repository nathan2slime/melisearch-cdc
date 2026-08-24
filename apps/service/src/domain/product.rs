#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Product {
    id: i32,
    name: String,
    description: Option<String>,
    price_cents: i32,
    stock: i32,
}

impl Product {
    pub fn new(
        id: i32,
        name: impl Into<String>,
        description: Option<String>,
        price_cents: i32,
        stock: i32,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            description,
            price_cents,
            stock,
        }
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn price_cents(&self) -> i32 {
        self.price_cents
    }

    pub fn stock(&self) -> i32 {
        self.stock
    }
}
