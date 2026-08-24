use rdkafka::{
    ClientConfig, Message, Offset, TopicPartitionList,
    consumer::{CommitMode, Consumer, StreamConsumer},
};
use serde::Deserialize;

use crate::{
    application::products::{ProductEventDelivery, ProductEventSource, ProductEventSourceError},
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

    async fn next_event(&self) -> Result<ProductEventDelivery<Self::Ack>, ProductEventSourceError> {
        let message = self.consumer.recv().await.map_err(to_source_error)?;
        let ack = KafkaProductEventAck {
            topic: message.topic().to_owned(),
            partition: message.partition(),
            offset: message.offset(),
        };
        let event = match message.payload() {
            Some(payload) => parse_product_event(payload)?,
            None => ProductEvent::Ignore,
        };

        Ok(ProductEventDelivery::new(event, ack))
    }

    async fn commit(&self, ack: Self::Ack) -> Result<(), ProductEventSourceError> {
        let mut offsets = TopicPartitionList::new();
        offsets
            .add_partition_offset(&ack.topic, ack.partition, Offset::Offset(ack.offset + 1))
            .map_err(to_source_error)?;

        self.consumer
            .commit(&offsets, CommitMode::Async)
            .map_err(to_source_error)
    }
}

pub struct KafkaProductEventAck {
    topic: String,
    partition: i32,
    offset: i64,
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
