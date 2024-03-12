use bollard::Docker;
use bollard::container::StartContainerOptions;
#[cfg(test)]
use bollard::container::ListContainersOptions;
use serde::Deserialize;
use serde::Serialize;
use tracing::trace;
#[cfg(test)]
use std::collections::HashMap;
use bollard::exec::StartExecResults;
use bollard::exec::CreateExecOptions;
use futures_util::StreamExt;


pub mod kafka_connect;
pub mod mysql;
pub mod containers;


const DEFAULT_MYSQL_DB: &str = "mydb";

use crate::stack::containers::create_testbed_network;
use crate::stack::containers::zookeeper::{ZOOKEEPER, get_zookeeper};
use crate::stack::containers::kafka::{KAFKA, get_kafka};
use crate::stack::containers::schema_registry::{REGISTRY, get_registry};
use crate::stack::containers::kafka_connect::{KAFKA_CONNECT, get_kafka_connect};
use crate::stack::containers::kui::{KUI, get_kui};
use crate::stack::containers::mysql::{MYSQL, get_mysql};
use crate::stack::kafka_connect::create_new_connector;
use crate::stack::kafka_connect::get_new_db_connector;
use tracing::{error, info};

use crate::stack::mysql::create_db_forcefully;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestBed {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Stack {}

impl Stack {
    /// Starts all the images; downloading and creating them if needed;
    /// akin to 'just start'. This creates a "base" testbed based
    /// on the files in the "testbed" directory
    ///
    /// This brings online:
    /// - Zookeeper
    /// - Kafka
    /// - Installs and configures Kafka Connect
    /// - Installs Debezium in Kafka Connect
    /// - Schema Registry
    /// - MySQL
    /// - Kafka UI
    pub async fn start() {
        let docker = Docker::connect_with_local_defaults().unwrap();

        info!("Create testbed network");
        create_testbed_network(&docker).await;

        let _ = get_zookeeper(&docker).await.unwrap();
        let _ = get_kafka(&docker).await.unwrap();
        let _ = get_registry(&docker).await.unwrap();
        let kc = get_kafka_connect(&docker).await.unwrap();
        let _ = get_kui(&docker).await.unwrap();
        let _ = get_mysql(&docker).await.unwrap();

        info!("Starting zookeeper");
        let _ = &docker
            .start_container(ZOOKEEPER, None::<StartContainerOptions<String>>)
            .await
            .unwrap();

        info!("Starting kafka");
        let _ = &docker
            .start_container(KAFKA, None::<StartContainerOptions<String>>)
            .await
            .unwrap();

        info!("Starting schema-registry");
        let _ = &docker
            .start_container(REGISTRY, None::<StartContainerOptions<String>>)
            .await
            .unwrap();

        info!("Starting kafka-connect");
        let _ = &docker
            .start_container(KAFKA_CONNECT, None::<StartContainerOptions<String>>)
            .await
            .unwrap();

        info!("Install debezium");
        configure_debezium(&docker, kc.id).await;

        info!("Starting kafka-ui");
        let _ = &docker
            .start_container(KUI, None::<StartContainerOptions<String>>)
            .await
            .unwrap();

        info!("Starting db");
        let _ = &docker
            .start_container(MYSQL, None::<StartContainerOptions<String>>)
            .await
            .unwrap();
    }

    /// Stops all the images
    pub async fn stop() {
        let docker = Docker::connect_with_local_defaults().unwrap();

        info!("Stop mysql");
        if let Err(e) = &docker.stop_container(MYSQL, None).await {
            error!("{e:?}");
        }

        info!("Stop kafka-ui");
        if let Err(e) = &docker.stop_container(KUI, None).await {
            error!("{e:?}");
        }

        info!("Stop kafka-connect");
        if let Err(e) = &docker.stop_container(KAFKA_CONNECT, None).await {
            error!("{e:?}");
        }

        info!("Stop schema-registry");
        if let Err(e) = &docker.stop_container(REGISTRY, None).await {
            error!("{e:?}");
        }

        info!("Stop kafka");
        if let Err(e) = &docker.stop_container(KAFKA, None).await {
            error!("{e:?}");
        }

        info!("Stop zookeeper");
        if let Err(e) = &docker.stop_container(ZOOKEEPER, None).await {
            error!("{e:?}");
        }
    }

    /// Deletes all the images
    pub async fn teardown() {
        let docker = Docker::connect_with_local_defaults().unwrap();

        info!("Remove mysql");
        if let Err(e) = &docker.remove_container(MYSQL, None).await {
            error!("{e:?}");
        }

        info!("Remove kafka-ui");
        if let Err(e) = &docker.remove_container(KUI, None).await {
            error!("{e:?}");
        }

        info!("Remove kafka-connect");
        if let Err(e) = &docker.remove_container(KAFKA_CONNECT, None).await {
            error!("{e:?}");
        }

        info!("Remove schema-registry");
        if let Err(e) = &docker.remove_container(REGISTRY, None).await {
            error!("{e:?}");
        }

        info!("Remove kafka");
        if let Err(e) = &docker.remove_container(KAFKA, None).await {
            error!("{e:?}");
        }

        info!("Remove zookeeper");
        if let Err(e) = &docker.remove_container(ZOOKEEPER, None).await {
            error!("{e:?}");
        }

        info!("Prune volumes");
        if let Err(e) = &docker.prune_volumes::<String>(None).await {
            error!("{e:?}");
        }


    }

    /// Stamps out a new "testbed" and returns the
    /// configuration for that testbed
    pub async fn new_testbed(test_name: &str) -> TestBed {
        let rng = rand::random::<u32>();
        let testbed_name = format!("{}_{}", test_name, rng);
        

        let docker = Docker::connect_with_local_defaults().unwrap();
        clone_database(&docker, None, &testbed_name).await;
        let json = &get_new_db_connector(None, &testbed_name);

        trace!("{json:?}");
        println!("{json:?}");

        create_new_connector(json).await;

        let testbed = TestBed {
            name: testbed_name,
        };

        trace!("{testbed:#?}");
        println!("{testbed:#?}");

        testbed
    }
}


#[tokio::test]
async fn create_testbed() {
    Stack::new_testbed("tester").await;
}


#[tokio::test]
async fn list_containers() {
    println!("list_containers");
    let docker = Docker::connect_with_local_defaults().unwrap();

    let mut filters = HashMap::new();
    filters.insert("health", vec!["healthy"]);

    let _options = Some(ListContainersOptions {
        all: true,
        filters,
        ..Default::default()
    });

    let o: Option<ListContainersOptions<String>> = None;

    let containers = docker.list_containers(o).await.unwrap();
    for c in containers {
        if let Some(ref names) = c.names {
            println!("Names: {names:?}");
            if names.iter().any(|n| n == MYSQL) {
                let id = c.id.clone().unwrap();
                println!("{}", id);
            }
        }
    }
}

pub async fn clone_database(docker: &Docker, original_db: Option<&str>, new_db: &str) {

    let source_db = match original_db {
        Some(db) => db.to_string(),
        None => DEFAULT_MYSQL_DB.to_string(),
    };

    info!("Cloning db: {} to: {}", source_db, new_db);
    println!("Cloning db: {} to: {}", source_db, new_db);

    info!("Force drop and create: {}", new_db);
    create_db_forcefully(new_db);

    //mysqldump db_name | mysql new_db_name
    //mysqldump -u <user name> --password=<pwd> <original db> | mysql -u <user name> -p <new db>

    let cmd = vec![
        "bash".to_string(),
        "-c".to_string(),
        format!(
            "mysqldump -ptestbed {} | mysql -ptestbed {}",
            source_db, new_db
        ),
    ];

    info!("Exec commands to clone...");
    for c in &cmd {
        info!(c);
    }

    // non interactive
    let exec = docker
        .create_exec(
            MYSQL,
            CreateExecOptions {
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                cmd: Some(cmd),
                ..Default::default()
            },
        )
        .await
        .unwrap()
        .id;

    if let StartExecResults::Attached { mut output, .. } =
        docker.start_exec(&exec, None).await.unwrap()
    {
        while let Some(Ok(msg)) = output.next().await {
            print!("{msg}");
        }
    } else {
        error!("Could not attach to kafka-connect container");
    }
}

#[tokio::test]
async fn test_clone_db() {
    println!("test_clone_db");
    let docker = Docker::connect_with_local_defaults().unwrap();
    clone_database(&docker, None, "foobar").await;
}

pub async fn configure_debezium(docker: &Docker, kafka_connect_id: String) {
    info!("Configuring debezium");

    let cmd = vec![
        "bash", 
        "-c", 
        "confluent-hub install --no-prompt debezium/debezium-connector-mysql:1.7.0",
        "confluent-hub install --no-prompt confluentinc/kafka-connect-jdbc:latest",
        "curl -o /usr/share/confluent-hub-components/confluentinc-kafka-connect-jdbc/lib/mysql.jar https://repo1.maven.org/maven2/mysql/mysql-connector-java/8.0.30/mysql-connector-java-8.0.30.jar",
        "/etc/confluent/docker/run sleep infinity"
        ];

    info!("Exec commands...");
    for c in &cmd {
        info!(c);
    }

    // non interactive
    let exec = docker
        .create_exec(
            &kafka_connect_id,
            CreateExecOptions {
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                cmd: Some(cmd),
                ..Default::default()
            },
        )
        .await
        .unwrap()
        .id;

    if let StartExecResults::Attached { mut output, .. } =
        docker.start_exec(&exec, None).await.unwrap()
    {
        while let Some(Ok(msg)) = output.next().await {
            print!("{msg}");
        }
    } else {
        error!("Could not attach to kafka-connect container");
    }
}

