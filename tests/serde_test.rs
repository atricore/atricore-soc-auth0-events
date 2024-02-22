use log::info;
use serde_json::{json, Value};
use std::fs;

mod common;

#[test]
fn serialize_payload_test() {
    common::setup();
    let payload_file = "./tests/data/payload.json";
    let payload = fs::read_to_string(payload_file).expect("Unable to read file payload file");

    info!("{}", payload);
    let record: Value = serde_json::from_str(&payload).expect("Unable to parse the JSON");

    let alert = json!({"auth0alert": record});

    info!(
        "{}",
        serde_json::to_string(&alert).expect("Unable to serilize to JSON")
    );
}

#[test]
fn serialize_multiple_payloads_test() {
    common::setup();
}
