use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

use std::{
    collections::HashMap, io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}
};

mod utils;
use utils::show_users;

pub fn run_server(address: &str, db: Vec<HashMap<&'static str, String>>) {
    let listener = TcpListener::bind(address).unwrap();
    let pool = ThreadPool::new(4);

    let db: Arc<Mutex<Vec<HashMap<&str, String>>>> = Arc::new(Mutex::new(db));

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let database = Arc::clone(&db);

        pool.execute(|| {
            handle_connection(stream, database);
        });
    }
}

fn handle_connection(mut stream: TcpStream, db: Arc<Mutex<Vec<HashMap<&str, String>>>>) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();
    let mut request_data = request_line.split(" ");
    let method = request_data.next().unwrap();
    let path = request_data.next().unwrap();

    let (status_line, contents) = match (method, path) {
        ("GET", "/users") => ("HTTP/1.1 200 OK", show_users(db)),
        _ => ("HTTP/1.1 404 NOT FOUND", "Not found".to_string()),
    };
    let length = contents.len();

    let response =
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender: Some(sender) }
    }
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");

                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });
        Worker { id, thread: Some(thread) }
    }
    
}
