use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::User;

#[derive(Debug, PartialEq)]
pub enum Errors {
    ServerError(u16),
    UserError(u16),
}

pub fn show_users(db: Arc<Mutex<Vec<User>>>) -> Result<String, Errors> {
    let users = db.lock().unwrap();
    let json = serde_json::to_string(&*users).map_err(|_| Errors::ServerError(500));
    json
}

pub fn show_user(db: Arc<Mutex<Vec<User>>>, id: u32) -> Result<String, Errors> {
    let users = db.lock().map_err(|_| Errors::ServerError(500))?;
    let user = users
        .iter()
        .find(|user| user.id == id)
        .ok_or(Errors::UserError(400))?;
    let json = serde_json::to_string(user).map_err(|_| Errors::ServerError(500));
    json
}

pub fn add_user(db: Arc<Mutex<Vec<User>>>, name: &str, lastname: &str) -> Result<String, Errors> {
    let mut users = db.lock().map_err(|_| Errors::ServerError(500))?;
    let last_user = users.last();
    let id = if let Some(last_user) = last_user {
        last_user.id + 1
    } else {
        0
    };
    let user = User {
        id,
        name: name.to_string(),
        lastname: lastname.to_string(),
    };
    users.push(user);

    Ok(format!("{}", id))
}

pub fn change_user_data(
    db: Arc<Mutex<Vec<User>>>,
    id: u32,
    change_data: HashMap<String, String>,
) -> Result<String, Errors> {
    let keys: Vec<_> = change_data.keys().collect();
    let mut users = db.lock().unwrap();
    let user = users
        .iter_mut()
        .find(|user| user.id == id)
        .ok_or(Errors::UserError(400))?;
    match keys[0].as_str() {
        "name" => user.name = change_data.get("name").unwrap().to_string(),
        "lastname" => user.lastname = change_data.get("lastname").unwrap().to_string(),
        _ => return Err(Errors::UserError(400)),
    };
    Ok("".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_db() -> (Vec<User>, Arc<Mutex<Vec<User>>>) {
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
        let db = Arc::new(Mutex::new(users.clone()));

        (users, db)
    }
    #[test]
    fn test_show_users() {
        let (users, db) = create_db();
        let output = show_users(db);
        let result: Vec<User> = serde_json::from_str(output.unwrap().as_str()).unwrap();
        assert_eq!(result, users);
    }

    #[test]
    fn test_show_user() {
        let (users, db) = create_db();
        let output = show_user(db, 1);
        let result: User = serde_json::from_str(output.unwrap().as_str()).unwrap();
        assert_eq!(result, users[0]);
    }

    #[test]
    fn test_adds_user_to_the_end() {
        let (_, db) = create_db();
        let output = add_user(db.clone(), "Test", "Test1");

        let new_users = db.lock().unwrap();
        let last_user = new_users.last().unwrap();
        assert_eq!(output.unwrap(), "3".to_string());
        assert_eq!(
            *last_user,
            User {
                id: 3,
                name: "Test".to_string(),
                lastname: "Test1".to_string()
            }
        );
    }

    #[test]
    fn test_adds_user_to_empty() {
        let users = Vec::new();
        let db = Arc::new(Mutex::new(users));
        let output = add_user(db.clone(), "Test", "Test1");

        let new_users = db.lock().unwrap();
        let last_user = new_users.last().unwrap();
        assert_eq!(output.unwrap(), "0".to_string());
        assert_eq!(
            *last_user,
            User {
                id: 0,
                name: "Test".to_string(),
                lastname: "Test1".to_string()
            }
        );
    }

    #[test]
    fn test_change_user_name() {
        let (_, db) = create_db();

        let change_data = HashMap::from([("name".to_string(), "Test".to_string())]);
        let _ = change_user_data(db.clone(), 1, change_data);

        let new_users = db.lock().unwrap();
        let last_user = new_users.get(0).unwrap();
        assert_eq!(
            *last_user,
            User {
                id: 1,
                name: "Test".to_string(),
                lastname: "Shutov".to_string()
            }
        );
    }

    #[test]
    fn test_change_user_lastname() {
        let (_, db) = create_db();
        let change_data = HashMap::from([("lastname".to_string(), "Test".to_string())]);
        let _ = change_user_data(db.clone(), 1, change_data);

        let new_users = db.lock().unwrap();
        let last_user = new_users.get(0).unwrap();
        assert_eq!(
            *last_user,
            User {
                id: 1,
                name: "Hlib".to_string(),
                lastname: "Test".to_string()
            }
        );
    }

    #[test]
    fn test_returns_error_if_not_exists() {
        let (_, db) = create_db();
        let change_data = HashMap::from([("lastname".to_string(), "Test".to_string())]);
        let output = change_user_data(db.clone(), 5, change_data);

        assert_eq!(output, Err(Errors::UserError(400)));
    }
}
