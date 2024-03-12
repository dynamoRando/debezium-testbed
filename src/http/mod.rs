use rocket::{get, post, Shutdown};
use tracing::info;
use crate::stack::{Stack, TestBed};
use rocket::serde::json::Json;


#[post("/testbed/<name>")]
pub async fn testbed(name: &str) -> Json<TestBed> {
    let testbed = Stack::new_testbed(name).await;
    Json(testbed)
}

#[get("/shutdown")]
pub async fn shutdown(shutdown: Shutdown) -> &'static str {
    info!("Shutting down");
    Stack::stop().await;
    Stack::teardown().await;
    shutdown.notify();
    "Shutting down..."
}