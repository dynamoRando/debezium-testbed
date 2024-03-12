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

pub const KAFKA_IMAGE: &str = "confluentinc/cp-kafka:latest";
pub const KAFKA: &str = "kafka";

pub async fn get_kafka(docker: &Docker) -> Result<ContainerCreateResponse> {
    info!("Getting kafka image");
    let _ = &docker
        .create_image(
            Some(CreateImageOptions {
                from_image: KAFKA_IMAGE,
                ..Default::default()
            }),
            None,
            None,
        )
        .try_collect::<Vec<_>>()
        .await?;

    //let mut exposed_ports = HashMap::new();
    //exposed_ports.insert("9092/tcp", HashMap::new());

    let first_binding = PortBinding {
        host_ip: None,
        host_port: Some("29092".to_string()),
    };

    let second_binding = PortBinding {
        host_ip: None,
        host_port: Some("9092".to_string()),
    };

    let mut portmap = PortMap::new();
    portmap.insert("9092/tcp".to_string(), vec![first_binding, second_binding].into());

    let kafka = Config {
        image: Some(KAFKA_IMAGE),
        cmd: Some(vec!["/etc/confluent/docker/run"]),
        env: Some(vec![
            "KAFKA_ZOOKEEPER_CONNECT=zookeeper:2181",
            "KAFKA_LISTENERS=INTERNAL://0.0.0.0:9092,OUTSIDE://0.0.0.0:29092",
            "KAFKA_ADVERTISED_LISTENERS=INTERNAL://kafka:9092,OUTSIDE://localhost:29092",
            "KAFKA_BROKER_ID=1",
            "KAFKA_LISTENER_SECURITY_PROTOCOL_MAP=INTERNAL:PLAINTEXT,OUTSIDE:PLAINTEXT",
            "KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR=1",
            "KAFKA_INTER_BROKER_LISTENER_NAME=INTERNAL",
        ]),
        host_config: Some(HostConfig {
            network_mode: Some(String::from(NETWORK)),
            port_bindings: Some(portmap),
            ..Default::default()
        }),
        //exposed_ports: Some(exposed_ports),
        ..Default::default()
    };

    info!("Creating kafka container");
    let container = &docker
        .create_container(
            Some(CreateContainerOptions {
                name: "kafka",
                platform: None,
            }),
            kafka,
        )
        .await?;

    Ok(container.clone())
}

