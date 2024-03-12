use std::{env, fs, path::{Path, PathBuf}};

use reqwest::header::{ACCEPT, CONTENT_TYPE};
use tracing::info;
use rand::Rng;
use crate::stack::DEFAULT_MYSQL_DB;


const KAFKA_CONNECT_URL: &str = "http://localhost:28083/connectors";
 

/*

command: "http://kafka-connect:8083/connectors -X POST -H 
'Content-Type: application/json' --data '@/debezium.json'" 

curl -X PUT http://localhost:28083/connector-plugins/io.debezium.connector.mysql.MySqlConnector/config/validate -H "Content-Type: application/json" -d '
{
    "connector.class": "io.debezium.connector.mysql.MySqlConnector",
    "database.user": "root",
    "topic.prefix": "dbz",
    "schema.history.internal.kafka.topic": "schema-changes.mydb.history",
    "database.server.id": "1",
    "tasks.max": "1",
    "database.hostname": "db",
    "database.password": "testbed",
    "name": "mydb",
    "schema.history.internal.kafka.bootstrap.servers": "kafka:9092",
    "database.port": "3306",
    "database.include.list": "mydb",
    "database.server.name": "db",
    "database.history.kafka.topic": "mydb.history",
    "database.history.kafka.bootstrap.servers": "kafka:9092"
}';

*/


/// POSTs the specified JSON to Kafka Connect to create a new Debezium connector
pub async fn create_new_connector(json: &str) {
    let client = reqwest::Client::new();
    let response = client
        .post(KAFKA_CONNECT_URL)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .body(json.to_string())
        .send()
        .await
        .unwrap();

    println!("{response:?}");

    info!("Response from Kafka Connect: {response:?}");
}

/// Returns JSON for creating a Debezium connector for the specified database. It does this by taking the JSON template
/// in your `dbz_init/dbz.json` and replacing the old or default database name with the new name provided
pub fn get_new_db_connector(old_db: Option<&str>, new_db: &str) -> String {

    let source_db = match old_db {
        Some(db) => db.to_string(),
        None => DEFAULT_MYSQL_DB.to_string(),
    };

    let json = get_debezium_connector_config();
    let json = json.replace(&source_db, new_db);

    let original_server_id = r#""database.server.id": "1","#;

    let mut rng = rand::thread_rng();
    let new_server_id = format!("\"database.server.id\": \"{}\",", rng.gen_range(0..1000));

    let content = json.replace(original_server_id, &new_server_id);

    let original_db_namespace = r#""database.server.name": "db","#;
    let new_server_namespace = format!("\"database.server.name\": \"{}\",", new_db);

    let content = content.replace(original_db_namespace, &new_server_namespace);

    println!("{content:#?}");

    content
}

fn get_debezium_connector_config() -> String {
    let dir = get_dbz_init_dir();
    let file = Path::join(&dir, "dbz.json");
    let json = fs::read_to_string(file).unwrap();

    info!("Original connector: {json:?}");

    json
}

fn get_dbz_init_dir() -> PathBuf {
    let path = env::current_dir().unwrap();
    path.join("dbz_init")
}

#[test]
fn test_parse_dbz() {
    get_debezium_connector_config();
}