use rust_api::{run_server, User};
use serde_json::json;
use std::{
    io::{Read, Write},
    net::TcpStream,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

fn create_users() -> Vec<User> {
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
    vec![user_1, user_2]
}

fn get_responce(
    address: &'static str,
    path: &str,
    method: &str,
    body: &str,
    db: Vec<User>,
) -> (String, String, Vec<User>) {
    let db = Arc::new(Mutex::new(db));
    let server_db = Arc::clone(&db);
    thread::spawn(|| {
        run_server(address, server_db);
    });

    thread::sleep(Duration::from_secs(1));

    let mut stream = TcpStream::connect(address).unwrap();
    let request = format!(
        "{} {} HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{}",
        method,
        path,
        body.len(),
        body
    );
    stream.write_all(request.as_bytes()).unwrap();

    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();

    let status_line = response.lines().next().unwrap();
    let response_data: Vec<&str> = status_line.split(" ").collect();
    let response_body: Vec<&str> = response.split("\r\n").collect();

    let db = db.lock().unwrap().to_vec();
    (
        response_data[1].to_string(),
        response_body.last().unwrap().to_string(),
        db,
    )
}

#[test]
fn test_empty_users() {
    let (code, response, _) = get_responce("127.0.0.1:7878", "/users", "GET", "", Vec::new());

    assert_eq!(code, "200".to_string());
    assert_eq!(response, "[]".to_string());
}

#[test]
fn test_show_users() {
    let users = create_users();
    let (code, response, _) = get_responce("127.0.0.1:7879", "/users", "GET", "", users.clone());

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
    let users = create_users();

    let (code, response, _) = get_responce("127.0.0.1:7880", "/users/1", "GET", "", users);

    let result: User = serde_json::from_str(response.as_str()).unwrap();

    assert_eq!(code, "200".to_string());
    assert_eq!(result, user_1);
}

#[test]
fn test_invalid_user_id() {
    let users = create_users();
    let (code, response, _) = get_responce("127.0.0.1:7881", "/users/test/", "GET", "", users);

    assert_eq!(code, "400".to_string());
    assert_eq!(response, "Invalid input");
}

#[test]
fn test_adding_user() {
    let users = create_users();

    let user_3 = User {
        id: 3,
        name: "Wojciech".to_string(),
        lastname: "Oczkowski".to_string(),
    };

    let body = json!({
        "name": "Wojciech",
        "lastname": "Oczkowski",
    })
    .to_string();

    let (code, response, db) =
        get_responce("127.0.0.1:7882", "/users", "POST", body.as_str(), users);

    assert_eq!(code, "201".to_string());
    assert_eq!(response, "3");
    assert_eq!(db[2], user_3);
}

#[test]
fn test_adding_user_invalid_data() {
    let users = create_users();

    let (code, response, _) = get_responce("127.0.0.1:7883", "/users", "POST", "test", users);

    assert_eq!(code, "400".to_string());
    assert_eq!(response, "Invalid input");
}
#[test]
fn test_change_user_name() {
    let users = create_users();

    let body = json!({
        "name": "Test",
    })
    .to_string();

    let (code, _, db) = get_responce("127.0.0.1:7884", "/users/1", "PATCH", body.as_str(), users);

    assert_eq!(code, "204".to_string());
    assert_eq!(
        *db.get(0).unwrap(),
        User {
            id: 1,
            name: "Test".to_string(),
            lastname: "Shutov".to_string()
        }
    );
}

#[test]
fn test_change_user_name_invalid_id() {
    let users = create_users();

    let body = json!({
        "name": "Test",
    })
    .to_string();

    let (code, response, _) =
        get_responce("127.0.0.1:7885", "/users/5", "PATCH", body.as_str(), users);

    assert_eq!(code, "400".to_string());
    assert_eq!(response, "Invalid input".to_string());
}

#[test]
fn test_change_user_name_invalid_body() {
    let users = create_users();

    let body = json!({
        "test": "Test",
    })
    .to_string();

    let (code, response, _) =
        get_responce("127.0.0.1:7886", "/users/5", "PATCH", body.as_str(), users);

    assert_eq!(code, "400".to_string());
    assert_eq!(response, "Invalid input".to_string());
}

#[test]
fn test_change_user_name_invalid_body_json() {
    let users = create_users();

    let (code, response, _) = get_responce("127.0.0.1:7887", "/users/5", "PATCH", "test", users);

    assert_eq!(code, "400".to_string());
    assert_eq!(response, "Invalid input".to_string());
}
