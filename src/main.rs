use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::{Header, Status};
use rocket::{launch, routes, Request, Response};
use tracing::info;
use crate::stack::Stack;
use crate::http::{testbed, shutdown};

pub mod stack;
pub mod http;

#[launch]
async fn rocket() -> _  {
    tracing_subscriber::fmt::init(); 

    info!("--- STARTUP DOCKER ---");
    Stack::stop().await;
    Stack::teardown().await;
    Stack::start().await;


    info!("--- STARTUP HTTP ---");
    rocket::build()
        .attach(CORS)
        .mount("/", routes![testbed, shutdown])
        
}


pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS, DELETE",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
        response.set_status(Status::Ok)
    }
}