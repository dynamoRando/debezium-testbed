use bollard::models::ContainerCreateResponse;
use bollard::Docker;
use bollard::image::CreateImageOptions;
use anyhow::Result;
use bollard::models::HostConfig;
use crate::stack::containers::NETWORK;
use bollard::container::Config;
use futures_util::TryStreamExt;
use tracing::info;
use bollard::container::CreateContainerOptions;
use bollard::models::{PortBinding, PortMap};

pub const KAFKA_CONNECT: &str = "kafka-connect";
pub const KAFKA_CONNECT_IMAGE: &str = "confluentinc/cp-kafka-connect-base:latest";

pub async fn get_kafka_connect(docker: &Docker) -> Result<ContainerCreateResponse> {
    info!("Getting kafka-connect image");
    let _ = &docker
        .create_image(
            Some(CreateImageOptions {
                from_image: KAFKA_CONNECT_IMAGE,
                ..Default::default()
            }),
            None,
            None,
        )
        .try_collect::<Vec<_>>()
        .await?;

    let binding = PortBinding {
        host_ip: None,
        host_port: Some("28083".to_string()),
    };
    let mut portmap = PortMap::new();
    portmap.insert("8083/tcp".to_string(), vec![binding].into());

    info!("Setup kafka-connect");
    let connect_config = Config {
        image: Some(KAFKA_CONNECT_IMAGE),
        env: Some(vec![
      "CONNECT_BOOTSTRAP_SERVERS=localhost:29092,kafka:9092",
      "CONNECT_REST_PORT=8083",
      "CONNECT_GROUP_ID=kafka-connect",
      "CONNECT_CONFIG_STORAGE_TOPIC=_connect-configs",
      "CONNECT_OFFSET_STORAGE_TOPIC=_connect-offsets",
      "CONNECT_STATUS_STORAGE_TOPIC=_connect-status",
      "CONNECT_KEY_CONVERTER=org.apache.kafka.connect.storage.StringConverter",
      "CONNECT_VALUE_CONVERTER=io.confluent.connect.avro.AvroConverter",
      "CONNECT_VALUE_CONVERTER_SCHEMA_REGISTRY_URL=http://schema-registry:8081",
      "CONNECT_REST_ADVERTISED_HOST_NAME=kafka-connect",
      "CONNECT_CONFIG_STORAGE_REPLICATION_FACTOR=1",
      "CONNECT_OFFSET_STORAGE_REPLICATION_FACTOR=1",
      "CONNECT_STATUS_STORAGE_REPLICATION_FACTOR=1",
      "CONNECT_PLUGIN_PATH: /usr/share/java,/usr/share/confluent-hub-components,/data/connect-jars"
         ]),
        host_config: Some(HostConfig {
            network_mode: Some(String::from(NETWORK)),
            port_bindings: Some(portmap),
            ..Default::default()
        }),
        ..Default::default()
    };

    info!("Creating kakfa-connect container");
    let container = &docker
        .create_container(
            Some(CreateContainerOptions {
                name: KAFKA_CONNECT,
                platform: None,
            }),
            connect_config,
        )
        .await?;

    Ok(container.clone())
}

