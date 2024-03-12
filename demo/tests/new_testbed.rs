use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use demo::mysql::{add_test_data, get_mysql_pool};
use demo::{TestBed, DEMO_SCHEMA_REGISTRY_URL};
use rdkafka::message::Message;
use schema_registry_converter::async_impl::avro::AvroDecoder;
use schema_registry_converter::async_impl::schema_registry::SrSettings;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

#[tokio::test(flavor = "multi_thread", worker_threads = 10)]
async fn test_testbed() {
    tracing_subscriber::fmt::init();
    let name = "demo";

    let testbed_url = format!("http://localhost:8000/testbed/{}", name);
    //let shutdown_url = "http://localhost:8000/shutdown/";

    // Get a new testbed by POST to the url with the testbed name
    let client = reqwest::Client::new();
    let json = client
        .post(testbed_url)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let testbed: TestBed = serde_json::from_str(&json).unwrap();
    let testbed_name = testbed.name;

    info!("Testbed name: {testbed_name:?}");

    // Add init test data
    let pool = get_mysql_pool(&testbed_name);
    add_test_data(pool.clone(), 0);

    let topic_name = format!("{}.{}.example", testbed_name, testbed_name);

    let testbed_has_messages = false;
    let testbed_has_messages = Arc::new(Mutex::new(testbed_has_messages));
    let cancellation_token = CancellationToken::new();

    // Poll kafka for data
    let has_messages = testbed_has_messages.clone();
    let token = cancellation_token.clone();
    let poll_handle = tokio::spawn(async move {
        let sr_settings = SrSettings::new(DEMO_SCHEMA_REGISTRY_URL.to_string());
        let decoder = AvroDecoder::new(sr_settings.clone());
        let consumer = demo::consumer::create_consumer_for(&topic_name);

        let total_seconds = 10;
        let mut seconds = 0;

        loop {
            info!("Sleeping polling thread for {seconds:?} of {total_seconds:?} seconds.");
            thread::sleep(Duration::from_secs(1));

            if token.is_cancelled() {
                break;
            }

            if seconds >= total_seconds {
                info!("Quitting because total seconds arrived");
                break;
            } else {
                seconds += 1;
            }

            match consumer.recv().await {
                Ok(message) => {
                    info!("Got message: {message:?}");

                    if let Some(payload) = message.payload() {
                        let value = decoder.decode(Some(payload)).await.unwrap();
                        let value = value.value;
                        info!("Got value: {value:?}");
                        *has_messages.lock().unwrap() = true;
                    }
                }
                Err(e) => {
                    error!("{e:?} - topic may have not been created yet");
                }
            }
        }
    });

    // Loop to let polling finish
    let total_seconds = 20;
    let mut seconds = 0;
    while seconds <= total_seconds {
        info!("Sleeping test thread for {seconds:?} of {total_seconds:?} seconds.");
        thread::sleep(Duration::from_secs(1));
        add_test_data(pool.clone(), seconds);
        seconds += 1;
        let had_messages = *testbed_has_messages.lock().unwrap();
        if had_messages {
            info!("Messages found, quitting.");
            cancellation_token.cancel();
            poll_handle.abort();
            break;
        }
    }

    let had_messages = *testbed_has_messages.lock().unwrap();
    info!("Had messages in testbed: {had_messages:?}");
    assert!(had_messages);
}
