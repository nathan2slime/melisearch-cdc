use std::{
    collections::{HashMap, hash_map::Entry},
    future::Future,
    time::Duration,
};

use async_trait::async_trait;
use tokio::time::sleep;

use crate::domain::product::{ProductDocument, ProductEvent};

pub struct ProductEventBatchDelivery<Ack> {
    events: Vec<ProductEvent>,
    ack: Ack,
}

impl<Ack> ProductEventBatchDelivery<Ack> {
    pub fn new(events: Vec<ProductEvent>, ack: Ack) -> Self {
        Self { events, ack }
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

    async fn next_batch(
        &self,
        max_size: usize,
        max_wait: Duration,
    ) -> Result<ProductEventBatchDelivery<Self::Ack>, ProductEventSourceError>;
    async fn commit(&self, ack: &Self::Ack) -> Result<(), ProductEventSourceError>;
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
    async fn upsert_products(&self, products: &[ProductDocument]) -> Result<(), ProductIndexError>;
    async fn delete_products(&self, ids: &[i32]) -> Result<(), ProductIndexError>;
}

pub async fn apply_product_events(
    index: &impl ProductIndex,
    events: Vec<ProductEvent>,
) -> Result<(), ProductIndexError> {
    let changes = product_index_changes(events);

    if !changes.upserts.is_empty() {
        index.upsert_products(&changes.upserts).await?;
    }

    if !changes.deletes.is_empty() {
        index.delete_products(&changes.deletes).await?;
    }

    Ok(())
}

pub async fn run_product_indexer(
    source: &impl ProductEventSource,
    index: &impl ProductIndex,
    batch_size: usize,
    batch_max_wait: Duration,
    retry_delay: Duration,
) -> Result<(), ProductIndexerError> {
    retry_until_ok("prepare product index", retry_delay, || async {
        index
            .ensure_ready()
            .await
            .map_err(ProductIndexerError::from)
    })
    .await;

    loop {
        let delivery = retry_until_ok("read product event batch", retry_delay, || async {
            source
                .next_batch(batch_size, batch_max_wait)
                .await
                .map_err(ProductIndexerError::from)
        })
        .await;

        retry_until_ok("apply product event batch", retry_delay, || async {
            apply_product_events(index, delivery.events.clone())
                .await
                .map_err(ProductIndexerError::from)
        })
        .await;

        retry_until_ok("commit product event offsets", retry_delay, || async {
            source
                .commit(&delivery.ack)
                .await
                .map_err(ProductIndexerError::from)
        })
        .await;
    }
}

pub async fn retry_until_ok<T, E, Operation, OperationFuture>(
    description: &str,
    retry_delay: Duration,
    mut operation: Operation,
) -> T
where
    E: std::fmt::Display,
    Operation: FnMut() -> OperationFuture,
    OperationFuture: Future<Output = Result<T, E>>,
{
    let mut attempt = 1_u64;

    loop {
        match operation().await {
            Ok(value) => return value,
            Err(error) => {
                log::warn!(
                    "indexer failed to {description} on attempt {attempt}: {error}; retrying in {} ms",
                    retry_delay.as_millis()
                );
                attempt = attempt.saturating_add(1);

                if !retry_delay.is_zero() {
                    sleep(retry_delay).await;
                }
            }
        }
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
struct ProductIndexChanges {
    upserts: Vec<ProductDocument>,
    deletes: Vec<i32>,
}

fn product_index_changes(events: Vec<ProductEvent>) -> ProductIndexChanges {
    let mut product_ids = Vec::new();
    let mut latest_events = HashMap::new();

    for event in events {
        match event {
            ProductEvent::Upsert(product) => match latest_events.entry(product.id) {
                Entry::Vacant(entry) => {
                    product_ids.push(product.id);
                    entry.insert(ProductEvent::Upsert(product));
                }
                Entry::Occupied(mut entry) => {
                    entry.insert(ProductEvent::Upsert(product));
                }
            },
            ProductEvent::Delete(id) => match latest_events.entry(id) {
                Entry::Vacant(entry) => {
                    product_ids.push(id);
                    entry.insert(ProductEvent::Delete(id));
                }
                Entry::Occupied(mut entry) => {
                    entry.insert(ProductEvent::Delete(id));
                }
            },
            ProductEvent::Ignore => {}
        }
    }

    let mut changes = ProductIndexChanges::default();

    for id in product_ids {
        match latest_events.remove(&id) {
            Some(ProductEvent::Upsert(product)) => changes.upserts.push(product),
            Some(ProductEvent::Delete(id)) => changes.deletes.push(id),
            Some(ProductEvent::Ignore) | None => {}
        }
    }

    changes
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    #[tokio::test]
    async fn retry_until_ok_retries_until_operation_succeeds() {
        let attempts = AtomicUsize::new(0);

        let result = retry_until_ok("test operation", Duration::ZERO, || {
            let attempt = attempts.fetch_add(1, Ordering::SeqCst);

            async move {
                if attempt < 2 {
                    Err("not ready")
                } else {
                    Ok("ready")
                }
            }
        })
        .await;

        assert_eq!(result, "ready");
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn compacts_product_events_to_latest_event_per_product() {
        let changes = product_index_changes(vec![
            ProductEvent::Upsert(ProductDocument::new(1, "Old Keyboard")),
            ProductEvent::Upsert(ProductDocument::new(2, "Mouse")),
            ProductEvent::Delete(1),
            ProductEvent::Ignore,
            ProductEvent::Upsert(ProductDocument::new(1, "Keyboard")),
        ]);

        assert_eq!(
            changes,
            ProductIndexChanges {
                upserts: vec![
                    ProductDocument::new(1, "Keyboard"),
                    ProductDocument::new(2, "Mouse"),
                ],
                deletes: vec![],
            }
        );
    }

    #[test]
    fn compacts_deleted_products_to_delete_batch() {
        let changes = product_index_changes(vec![
            ProductEvent::Upsert(ProductDocument::new(1, "Keyboard")),
            ProductEvent::Delete(1),
            ProductEvent::Delete(2),
        ]);

        assert_eq!(
            changes,
            ProductIndexChanges {
                upserts: vec![],
                deletes: vec![1, 2],
            }
        );
    }
}
