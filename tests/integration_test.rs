use a3c_soc_auth0_events_encode_lib::log4rs::{Log4rsEncoder, Log4rsEncoderSettings};
use a3c_soc_auth0_events_encode_lib::{Encode, EncoderType, EncodersSettings};
use actix_web::{test, web::Data, App};
use log::info;
use serde_json::Value;
use std::sync::Arc;
use std::{fs, future::Future};

mod common;

async fn payload_apply<F, Fut>(payload_file: &str, f: F) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn(Value) -> Fut,
    Fut: Future<Output = Result<(), Box<dyn std::error::Error>>>,
{
    // Read payloads from JSON file.
    let payloads = fs::read_to_string(payload_file).expect("Unable to read file payload file");
    let payloads_json: Vec<Value> =
        serde_json::from_str(&payloads).expect("Unable to parse the JSON");

    for payload in payloads_json {
        f(payload).await?;
    }
    Ok(())
}

async fn auth0_receiver_test<'a>(v: Value) -> Result<(), Box<dyn std::error::Error>> {
    // Do something asynchronous here
    // Create service
    info!("auth0_receiver test");

    let s = EncodersSettings {
        encoders: vec![EncoderType::Log4rs(Log4rsEncoderSettings {
            enable: Some(true),
            logger_name: "data_encoder".to_string().into_boxed_str(),
        })],
    };

    let e = create_log4rs_encoder(&s).unwrap();

    // Build test servcie w/app data
    let encode_arc: Arc<dyn Encode> = Arc::new(e.clone());
    let encode_data: Data<dyn Encode> = Data::from(encode_arc);

    let svc = test::init_service(App::new().app_data(encode_data).service(webhook_receiver)).await;

    // Create POST request using a JSON payload
    let req = test::TestRequest::post()
        .uri("/a3c-soc-auth0")
        .set_json(&v)
        .to_request();

    // Call the service
    let resp = test::call_service(&svc, req).await;

    info!("STATUS: {}", resp.status());
    assert!(resp.status().is_success());
    Ok(())
}

pub fn create_log4rs_encoder(
    settings: &EncodersSettings,
) -> Result<Log4rsEncoder, Box<dyn std::error::Error>> {
    // For now, build a log4rs encoder
    let es = settings.encoders.iter().find_map(|e| match e {
        EncoderType::Log4rs(l) => Some(l),
    });

    // if es is None -> Err
    let log4rs_settings = es.ok_or(anyhow::anyhow!("no log4rs encode configured"))?;
    if !log4rs_settings.enable.unwrap_or(false) {
        Err(anyhow::anyhow!("log4rs encoder not enabled"))?;
    };

    Ok(Log4rsEncoder::new(log4rs_settings.to_owned()))
}

#[actix_web::test]
async fn test_webhook_receiver() {
    common::setup();
    payload_apply("./tests/data/payloads.json", auth0_receiver_test)
        .await
        .expect("error running test_webhook_receiver")
}
