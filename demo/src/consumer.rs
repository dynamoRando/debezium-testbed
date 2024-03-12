use rdkafka::{
    config::RDKafkaLogLevel,
    consumer::{Consumer, StreamConsumer},
    ClientConfig,
};

use crate::DEMO_BROKER_URL;

//const GROUP_ID: &str = "demo";

pub fn create_consumer_for(topic_name: &str) -> StreamConsumer {
    let mut config = ClientConfig::new();

    config
        .set("group.id", topic_name)
        .set("bootstrap.servers", DEMO_BROKER_URL)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set("security.protocol", "plaintext")
        .set("auto.offset.reset", "earliest")
        .set_log_level(RDKafkaLogLevel::Debug);

    let consumer = config.create::<StreamConsumer>().unwrap();

    consumer.subscribe(&[topic_name]).unwrap();

    consumer
}
