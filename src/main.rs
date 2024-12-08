use std::{
    collections::HashMap, io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}
};

use std::sync::{Arc, Mutex};

use rust_api::ThreadPool;

mod utils;
use utils::show_users;

fn run_server(address: &str) {
    let listener = TcpListener::bind(address).unwrap();
    let pool = ThreadPool::new(4);

    let db: Arc<Mutex<Vec<HashMap<&str, String>>>> = Arc::new(Mutex::new(Vec::new()));

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let database = Arc::clone(&db);

        pool.execute(|| {
            handle_connection(stream, database);
        });
    }
}

fn main() {
    run_server("127.0.0.1:7878");
}

fn handle_connection(mut stream: TcpStream, db: Arc<Mutex<Vec<HashMap<&str, String>>>>) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();
    let mut request_data = request_line.split(" ");
    let method = request_data.next().unwrap();
    let path = request_data.next().unwrap();

    let (status_line, contents) = match (method, path) {
        ("GET", "/users") => {
            ("HTTP/1.1 200 OK", show_users(db))
        },
        _ => ("HTTP/1.1 404 NOT FOUND", "Not found".to_string()),
    };
    let length = contents.len();

    let response =
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}
