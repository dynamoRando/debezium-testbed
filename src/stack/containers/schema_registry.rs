use bollard::models::ContainerCreateResponse;
use bollard::Docker;
use bollard::image::CreateImageOptions;
use anyhow::Result;
use bollard::models::HostConfig;
use bollard::container::Config;
use futures_util::TryStreamExt;
use tracing::info;
use bollard::container::CreateContainerOptions;
use bollard::models::{PortBinding, PortMap};

pub const SCHEMA_REGISTRY_IMAGE: &str = "confluentinc/cp-schema-registry:7.5.1";
pub const REGISTRY: &str = "schema-registry";


pub async fn get_registry(docker: &Docker) -> Result<ContainerCreateResponse> {
    info!("Getting schema-registry image");
    let _ = &docker
        .create_image(
            Some(CreateImageOptions {
                from_image: SCHEMA_REGISTRY_IMAGE,
                ..Default::default()
            }),
            None,
            None,
        )
        .try_collect::<Vec<_>>()
        .await?;

      let binding = PortBinding {
        host_ip: None,
        host_port: Some("28081".to_string()),
    };

    let mut portmap = PortMap::new();
    portmap.insert("8081/tcp".to_string(), vec![binding].into());

    let registry_config = Config {
        image: Some(SCHEMA_REGISTRY_IMAGE),
        env: Some(vec![
            "SCHEMA_REGISTRY_HOST_NAME=schema-registry",
            "SCHEMA_REGISTRY_KAFKASTORE_BOOTSTRAP_SERVERS=PLAINTEXT://localhost:29092,PLAINTEXT://kafka:9092",
        ]),
        host_config: Some(HostConfig {
            network_mode: Some(String::from("testbed")),
            port_bindings: Some(portmap),
            ..Default::default()
        }),
        ..Default::default()
    };

    info!("Creating schema-registry container");
    let container = &docker
        .create_container(
            Some(CreateContainerOptions {
                name: "schema-registry",
                platform: None,
            }),
            registry_config,
        )
        .await?;

    Ok(container.clone())
}
