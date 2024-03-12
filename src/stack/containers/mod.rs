use bollard::Docker;
use bollard::network::CreateNetworkOptions;
use tracing::error;


pub mod mysql;
pub mod kafka;
pub mod kafka_connect;
pub mod zookeeper;
pub mod kui;
pub mod schema_registry;

pub const NETWORK: &str = "testbed";


pub async fn create_testbed_network(docker: &Docker) {
    let config = CreateNetworkOptions {
        name: NETWORK,
        check_duplicate: true,
        internal: false,
        ..Default::default()
    };

    if let Err(e) = docker.create_network(config).await {
        error!("{e:?}");
    }
}