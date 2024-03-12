use serde::{Serialize, Deserialize};

pub mod consumer; 
pub mod mysql;
pub const DEMO_BROKER_URL: &str = "localhost:29092";
pub const DEMO_SCHEMA_REGISTRY_URL: &str = "http://localhost:28081";


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestBed {
    pub name: String,
}