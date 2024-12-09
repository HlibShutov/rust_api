use rust_api::{run_server, User};
use std::{
    io::{Read, Write},
    net::TcpStream,
    thread,
    time::Duration,
};

fn get_responce(address: &'static str, path: &'static str, db: Vec<User>) -> (String, String) {
    thread::spawn(|| {
        run_server(address, db);
    });

    thread::sleep(Duration::from_secs(1));

    let mut stream = TcpStream::connect(address).unwrap();
    let request = format!("GET {} HTTP/1.1\r\nHost: localhost\r\n\r\n", path);
    stream.write_all(request.as_bytes()).unwrap();

    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();

    let status_line = response.lines().next().unwrap();
    let response_data: Vec<&str> = status_line.split(" ").collect();
    let response_body: Vec<&str> = response.split("\r\n").collect();

    (
        response_data[1].to_string(),
        response_body.last().unwrap().to_string(),
    )
}

#[test]
fn test_empty_users() {
    let (code, response) = get_responce("127.0.0.1:7878", "/users", Vec::new());

    assert_eq!(code, "200".to_string());
    assert_eq!(response, "[]".to_string());
}

#[test]
fn test_show_users() {
    let user_1 = User {
        id: 1,
        name: "Hlib".to_string(),
        lastname: "Shutov".to_string(),
    };
    let user_2 = User {
        id: 2,
        name: "Wojciech".to_string(),
        lastname: "Oczkowski".to_string(),
    };
    let users = vec![user_1, user_2];

    let (code, response) = get_responce("127.0.0.1:7879", "/users", users.clone());

    let result: Vec<User> = serde_json::from_str(response.as_str()).unwrap();

    assert_eq!(code, "200".to_string());
    assert_eq!(result, users);
}

#[test]
fn test_show_user() {
    let user_1 = User {
        id: 1,
        name: "Hlib".to_string(),
        lastname: "Shutov".to_string(),
    };
    let user_2 = User {
        id: 2,
        name: "Wojciech".to_string(),
        lastname: "Oczkowski".to_string(),
    };
    let users = vec![user_1.clone(), user_2];

    let (code, response) = get_responce("127.0.0.1:7880", "/users/1", users.clone());

    let result: User = serde_json::from_str(response.as_str()).unwrap();

    assert_eq!(code, "200".to_string());
    assert_eq!(result, user_1);
}

#[test]
fn test_invalid_user_id() {
    let user_1 = User {
        id: 1,
        name: "Hlib".to_string(),
        lastname: "Shutov".to_string(),
    };
    let user_2 = User {
        id: 2,
        name: "Wojciech".to_string(),
        lastname: "Oczkowski".to_string(),
    };
    let users = vec![user_1.clone(), user_2];

    let (code, response) = get_responce("127.0.0.1:7881", "/users/test/", users.clone());

    assert_eq!(code, "400".to_string());
    assert_eq!(response, "Invalid user ID");
}
