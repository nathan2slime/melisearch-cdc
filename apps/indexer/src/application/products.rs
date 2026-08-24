use async_trait::async_trait;

use crate::domain::product::{ProductDocument, ProductEvent};

pub struct ProductEventDelivery<Ack> {
    event: ProductEvent,
    ack: Ack,
}

impl<Ack> ProductEventDelivery<Ack> {
    pub fn new(event: ProductEvent, ack: Ack) -> Self {
        Self { event, ack }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ProductEventSourceError {
    Infrastructure(String),
}

impl std::fmt::Display for ProductEventSourceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Infrastructure(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for ProductEventSourceError {}

#[async_trait]
pub trait ProductEventSource {
    type Ack: Send;

    async fn next_event(&self) -> Result<ProductEventDelivery<Self::Ack>, ProductEventSourceError>;
    async fn commit(&self, ack: Self::Ack) -> Result<(), ProductEventSourceError>;
}

#[derive(Debug, Eq, PartialEq)]
pub enum ProductIndexError {
    Infrastructure(String),
}

impl std::fmt::Display for ProductIndexError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Infrastructure(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for ProductIndexError {}

#[derive(Debug, Eq, PartialEq)]
pub enum ProductIndexerError {
    Source(ProductEventSourceError),
    Index(ProductIndexError),
}

impl std::fmt::Display for ProductIndexerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Source(error) => write!(formatter, "product event source error: {error}"),
            Self::Index(error) => write!(formatter, "product index error: {error}"),
        }
    }
}

impl std::error::Error for ProductIndexerError {}

impl From<ProductEventSourceError> for ProductIndexerError {
    fn from(error: ProductEventSourceError) -> Self {
        Self::Source(error)
    }
}

impl From<ProductIndexError> for ProductIndexerError {
    fn from(error: ProductIndexError) -> Self {
        Self::Index(error)
    }
}

#[async_trait]
pub trait ProductIndex {
    async fn ensure_ready(&self) -> Result<(), ProductIndexError>;
    async fn upsert_product(&self, product: &ProductDocument) -> Result<(), ProductIndexError>;
    async fn delete_product(&self, id: i32) -> Result<(), ProductIndexError>;
}

pub async fn apply_product_event(
    index: &impl ProductIndex,
    event: ProductEvent,
) -> Result<(), ProductIndexError> {
    match event {
        ProductEvent::Upsert(product) => index.upsert_product(&product).await,
        ProductEvent::Delete(id) => index.delete_product(id).await,
        ProductEvent::Ignore => Ok(()),
    }
}

pub async fn run_product_indexer(
    source: &impl ProductEventSource,
    index: &impl ProductIndex,
) -> Result<(), ProductIndexerError> {
    index.ensure_ready().await?;

    loop {
        let delivery = source.next_event().await?;
        apply_product_event(index, delivery.event).await?;
        source.commit(delivery.ack).await?;
    }
}
