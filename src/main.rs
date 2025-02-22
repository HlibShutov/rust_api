use rust_api::{db_object::DataBase, run_server};
use std::sync::{Arc, Mutex};

fn main() {
    let db = Arc::new(Mutex::new(DataBase::new()));

    run_server("127.0.0.1:7878", db);
}
