
use bollard::models::ContainerCreateResponse;
use bollard::Docker;
use bollard::image::CreateImageOptions;
use anyhow::Result;
use bollard::models::HostConfig;
use bollard::container::Config;
use futures_util::TryStreamExt;
use tracing::info;
use bollard::container::CreateContainerOptions;
use crate::stack::containers::NETWORK;

pub const ZOOKEEPER: &str = "zookeeper";
pub const ZOOKEEPER_IMAGE: &str = "confluentinc/cp-zookeeper:latest";

pub async fn get_zookeeper(docker: &Docker) -> Result<ContainerCreateResponse> {
    info!("Getting zookeeper image");
    let _ = &docker
        .create_image(
            Some(CreateImageOptions {
                from_image: ZOOKEEPER_IMAGE,
                ..Default::default()
            }),
            None,
            None,
        )
        .try_collect::<Vec<_>>()
        .await?;

    let zookeeper_config = Config {
        image: Some(ZOOKEEPER_IMAGE),
        env: Some(vec![
            "ZOOKEEPER_CLIENT_PORT=2181",
            "ZOOKEEPER_TICK_TIME=2000",
            "ZOOKEEPER_SYNC_LIMIT=2",
        ]),
        host_config: Some(HostConfig {
            network_mode: Some(String::from(NETWORK)),
            ..Default::default()
        }),
        ..Default::default()
    };

    info!("Creating zookeeper container");
    let container = &docker
        .create_container(
            Some(CreateContainerOptions {
                name: ZOOKEEPER,
                platform: None,
            }),
            zookeeper_config,
        )
        .await?;

    Ok(container.clone())
}
