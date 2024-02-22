use log::info;
use log4rs;

use std::sync::Once;

static INIT: Once = Once::new();

pub fn setup() {
    INIT.call_once(|| {
        init_test_log();
    });
    info!("Test suite initialized");
}

fn init_test_log() {
    println!("initializing log4rs for testing ...");
    let fname = "./tests/log4rs-test.yml";

    println!("using log4rs config: {}", fname);
    let _ = log4rs::init_file(fname, Default::default())
        .map_err(|e| {
            panic!("Cannot initialize log: {} {}", fname, e.to_string());
        })
        .map(|_| ());
}
