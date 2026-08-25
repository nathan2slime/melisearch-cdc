use std::{collections::HashMap, time::Duration};

use rdkafka::{
    ClientConfig, Message, Offset, TopicPartitionList,
    consumer::{CommitMode, Consumer, StreamConsumer},
};
use serde::Deserialize;
use tokio::time::timeout;

use crate::{
    application::products::{
        ProductEventBatchDelivery, ProductEventSource, ProductEventSourceError,
    },
    domain::product::{ProductDocument, ProductEvent},
};

pub struct KafkaProductEventSource {
    consumer: StreamConsumer,
}

impl KafkaProductEventSource {
    pub fn new(
        bootstrap_servers: &str,
        group_id: &str,
        topic: String,
    ) -> Result<Self, ProductEventSourceError> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", bootstrap_servers)
            .set("group.id", group_id)
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "earliest")
            .create()
            .map_err(to_source_error)?;
        consumer.subscribe(&[&topic]).map_err(to_source_error)?;

        Ok(Self { consumer })
    }
}

#[async_trait::async_trait]
impl ProductEventSource for KafkaProductEventSource {
    type Ack = KafkaProductEventAck;

    async fn next_batch(
        &self,
        max_size: usize,
        max_wait: Duration,
    ) -> Result<ProductEventBatchDelivery<Self::Ack>, ProductEventSourceError> {
        let max_size = max_size.max(1);
        let mut events = Vec::with_capacity(max_size);
        let mut ack = KafkaProductEventAck::default();

        let message = self.consumer.recv().await.map_err(to_source_error)?;
        collect_message(&mut events, &mut ack, &message)?;

        while events.len() < max_size {
            let message = match timeout(max_wait, self.consumer.recv()).await {
                Ok(Ok(message)) => message,
                Ok(Err(error)) => return Err(to_source_error(error)),
                Err(_) => break,
            };

            collect_message(&mut events, &mut ack, &message)?;
        }

        Ok(ProductEventBatchDelivery::new(events, ack))
    }

    async fn commit(&self, ack: &Self::Ack) -> Result<(), ProductEventSourceError> {
        let mut offsets = TopicPartitionList::new();

        for ((topic, partition), offset) in &ack.offsets {
            offsets
                .add_partition_offset(topic, *partition, Offset::Offset(*offset + 1))
                .map_err(to_source_error)?;
        }

        self.consumer
            .commit(&offsets, CommitMode::Async)
            .map_err(to_source_error)
    }
}

#[derive(Default)]
pub struct KafkaProductEventAck {
    offsets: HashMap<(String, i32), i64>,
}

impl KafkaProductEventAck {
    fn record_message(&mut self, message: &impl Message) {
        self.offsets
            .entry((message.topic().to_owned(), message.partition()))
            .and_modify(|offset| *offset = (*offset).max(message.offset()))
            .or_insert(message.offset());
    }
}

fn collect_message(
    events: &mut Vec<ProductEvent>,
    ack: &mut KafkaProductEventAck,
    message: &impl Message,
) -> Result<(), ProductEventSourceError> {
    ack.record_message(message);

    let event = match message.payload() {
        Some(payload) => parse_product_event(payload)?,
        None => ProductEvent::Ignore,
    };
    events.push(event);

    Ok(())
}

fn to_source_error(error: rdkafka::error::KafkaError) -> ProductEventSourceError {
    ProductEventSourceError::Infrastructure(error.to_string())
}

fn json_error(error: serde_json::Error) -> ProductEventSourceError {
    ProductEventSourceError::Infrastructure(error.to_string())
}

fn missing_debezium_field(field: &'static str) -> ProductEventSourceError {
    ProductEventSourceError::Infrastructure(format!("Debezium product event is missing '{field}'"))
}

#[derive(Debug, Deserialize)]
struct DebeziumEnvelope {
    payload: Option<DebeziumPayload>,
}

#[derive(Debug, Deserialize)]
struct DebeziumPayload {
    before: Option<DebeziumProductDocument>,
    after: Option<DebeziumProductDocument>,
    op: String,
}

#[derive(Debug, Deserialize)]
struct DebeziumProductDocument {
    id: i32,
    name: String,
}

impl From<DebeziumProductDocument> for ProductDocument {
    fn from(product: DebeziumProductDocument) -> Self {
        Self::new(product.id, product.name)
    }
}

fn parse_product_event(payload: &[u8]) -> Result<ProductEvent, ProductEventSourceError> {
    let envelope = serde_json::from_slice::<DebeziumEnvelope>(payload).map_err(json_error)?;
    let Some(payload) = envelope.payload else {
        return Ok(ProductEvent::Ignore);
    };

    match payload.op.as_str() {
        "c" | "r" | "u" => payload
            .after
            .map(ProductDocument::from)
            .map(ProductEvent::Upsert)
            .ok_or(missing_debezium_field("after")),
        "d" => payload
            .before
            .map(|product| ProductEvent::Delete(product.id))
            .ok_or(missing_debezium_field("before")),
        _ => Ok(ProductEvent::Ignore),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_debezium_create_event_as_upsert() {
        let event = parse_product_event(
            br#"{
                "payload": {
                    "before": null,
                    "after": {
                        "id": 1,
                        "name": "Keyboard",
                        "description": "Mechanical",
                        "price_cents": 12999,
                        "stock": 7
                    },
                    "op": "c"
                }
            }"#,
        )
        .unwrap();

        assert_eq!(
            event,
            ProductEvent::Upsert(ProductDocument::new(1, "Keyboard"))
        );
    }

    #[test]
    fn parses_debezium_delete_event_as_delete() {
        let event = parse_product_event(
            br#"{
                "payload": {
                    "before": {
                        "id": 1,
                        "name": "Keyboard"
                    },
                    "after": null,
                    "op": "d"
                }
            }"#,
        )
        .unwrap();

        assert_eq!(event, ProductEvent::Delete(1));
    }

    #[test]
    fn ignores_debezium_tombstone_envelope() {
        let event = parse_product_event(br#"{"payload": null}"#).unwrap();

        assert_eq!(event, ProductEvent::Ignore);
    }
}
