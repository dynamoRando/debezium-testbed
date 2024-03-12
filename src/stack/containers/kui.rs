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


pub const KUI: &str = "kui";
pub const KUI_IMAGE: &str = "provectuslabs/kafka-ui:latest";

pub async fn get_kui(docker: &Docker) -> Result<ContainerCreateResponse> {
    info!("Getting kui image");
    let _ = &docker
        .create_image(
            Some(CreateImageOptions {
                from_image: KUI_IMAGE,
                ..Default::default()
            }),
            None,
            None,
        )
        .try_collect::<Vec<_>>()
        .await?;

    // exposed ports
    // An object mapping ports to an empty object in the form:  `{\"<port>/<tcp|udp|sctp>\": {}}`
    // expected struct `std::collections::HashMap<&str, std::collections::HashMap<(), ()>>`
    //let mut exposed_ports = HashMap::new();
    //exposed_ports.insert("8099/tcp", HashMap::new());

    let binding = PortBinding {
        host_ip: None,
        host_port: Some("8099".to_string()),
    };

    // PortMap describes the mapping of container ports to host ports, using the container's port-number and protocol as key in the format `<port>/<protocol>`,
    // for example, `80/udp`.  If a container's port is mapped for multiple protocols, separate entries are added to the mapping table.
    // special-casing PortMap, cos swagger-codegen doesn't figure out this type
    let mut portmap = PortMap::new();
    portmap.insert("8080/tcp".to_string(), vec![binding].into());

    let kui_config = Config {
        image: Some(KUI_IMAGE),
        env: Some(vec![
            "KAFKA_CLUSTERS_0_NAME=testbed",
            "KAFKA_CLUSTERS_0_BOOTSTRAPSERVERS=kafka:9092",
            "KAFKA_CLUSTERS_0_SCHEMAREGISTRY=http://schema-registry:8081",
            "KAFKA_CLUSTERS_0_KAFKACONNECT_0_NAME=connect-cluster",
            "KAFKA_CLUSTERS_0_KAFKACONNECT_0_ADDRESS=http://kafka-connect:8083",
            //KAFKA_CLUSTERS_0_KSQLDBSERVER: http://ksqldb-server-1:8088
            "DYNAMIC_CONFIG_ENABLED=true",
            "SERVER_PORT=8080",
        ]),
        host_config: Some(HostConfig {
            network_mode: Some(String::from(NETWORK)),
            //publish_all_ports: Some(true),
            port_bindings: Some(portmap),
            ..Default::default()
        }),
        //exposed_ports: Some(exposed_ports),
        ..Default::default()
    };

    info!("Creating kui container");
    let container = &docker
        .create_container(
            Some(CreateContainerOptions {
                name: KUI,
                platform: None,
            }),
            kui_config,
        )
        .await?;

    Ok(container.clone())
}
