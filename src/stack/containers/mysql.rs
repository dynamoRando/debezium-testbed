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
use bollard::models::{PortBinding, PortMap, Mount, MountTypeEnum};
use std::{env, path::PathBuf};


pub const MYSQL_IMAGE: &str = "mysql:8.0";
pub const MYSQL: &str = "db";
pub const MYSQL_INIT_PATH: &str = "/docker-entrypoint-initdb.d";
//const MYSQL_LOCAL_INIT_PATH: &str = "sql_init";

fn get_sql_init_dir() -> PathBuf {
    let path = env::current_dir().unwrap();
    path.join("sql_init")
}


pub async fn get_mysql(docker: &Docker) -> Result<ContainerCreateResponse> {
    info!("Getting db image");
    let _ = &docker
        .create_image(
            Some(CreateImageOptions {
                from_image: MYSQL_IMAGE,
                ..Default::default()
            }),
            None,
            None,
        )
        .try_collect::<Vec<_>>()
        .await?;

    let binding = PortBinding {
        host_ip: None,
        host_port: Some("23306".to_string()),
    };

    let mut portmap = PortMap::new();
    portmap.insert("3306/tcp".to_string(), vec![binding].into());

    let init_dir = get_sql_init_dir();
    let init_dir = init_dir.into_os_string().into_string().unwrap();
    //let init_dir = init_dir.replace('\"', "");
    //let init_dir = MYSQL_LOCAL_INIT_PATH.to_string();

    info!("Init dir: {init_dir:?}");

    let db_config = Config {
        image: Some(MYSQL_IMAGE),
        env: Some(vec![
            "MYSQL_USER=testbed",
            "MYSQL_PASSWORD=testbed",
            "MYSQL_DATABASE=testbed",
            "MYSQL_ROOT_PASSWORD=testbed",
        ]),
        host_config: Some(HostConfig {
            network_mode: Some(String::from(NETWORK)),
            port_bindings: Some(portmap),
            mounts: Some(vec![Mount {
                typ: Some(MountTypeEnum::BIND),
                source: Some(init_dir),
                target: Some(String::from(MYSQL_INIT_PATH)),
                ..Default::default()
            }]),
            ..Default::default()
        }),
        ..Default::default()
    };

    info!("Creating db container");
    let container = &docker
        .create_container(
            Some(CreateContainerOptions {
                name: MYSQL,
                platform: None,
            }),
            db_config,
        )
        .await?;

    Ok(container.clone())
}
