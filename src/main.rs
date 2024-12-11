use rust_api::run_server;
use std::sync::{Arc, Mutex};

fn main() {
    let db = Arc::new(Mutex::new(Vec::new()));

    run_server("127.0.0.1:7878", db);
}
