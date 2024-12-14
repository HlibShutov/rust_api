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

pub fn add_user(
    db: Arc<Mutex<Vec<User>>>,
    data: HashMap<String, String>,
    new_id: Option<u32>,
) -> Result<String, Errors> {
    let mut users = db.lock().map_err(|_| Errors::ServerError(500))?;
    let last_user = users.last();
    let id = new_id.unwrap_or(if let Some(last_user) = last_user {
        last_user.id + 1
    } else {
        0
    });

    if data.contains_key("name") && data.contains_key("lastname") {
        let user = User {
            id,
            name: data.get("name").unwrap().to_string(),
            lastname: data.get("lastname").unwrap().to_string(),
        };
        users.push(user);

        Ok(format!("{}", id))
    } else {
        Err(Errors::UserError(400))
    }
}

pub fn change_user_data(
    db: Arc<Mutex<Vec<User>>>,
    id: u32,
    change_data: HashMap<String, String>,
) -> Result<String, Errors> {
    let keys: Vec<_> = change_data.keys().collect();
    let mut users = db.lock().map_err(|_| Errors::ServerError(500))?;
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

pub fn add_or_modify_user(
    db: Arc<Mutex<Vec<User>>>,
    id: u32,
    data: HashMap<String, String>,
) -> Result<String, Errors> {
    if data.contains_key("name") && data.contains_key("lastname") {
        {
            let mut users = db.lock().map_err(|_| Errors::ServerError(500))?;

            if let Some(user) = users.iter_mut().find(|user| user.id == id) {
                user.name = data.get("name").unwrap().to_string();
                user.lastname = data.get("lastname").unwrap().to_string();
                return Ok("Modified user".to_string());
            }
        }

        add_user(db, data, Some(id))?;
        return Ok("Created new user".to_string());
    } else {
        Err(Errors::UserError(400))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_user(id: u32) -> User {
        User {
            id,
            name: "test".to_string(),
            lastname: "test1".to_string(),
        }
    }
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
        let data = HashMap::from([
            ("name".to_string(), "test".to_string()),
            ("lastname".to_string(), "test1".to_string()),
        ]);
        let output = add_user(db.clone(), data, None);

        let new_users = db.lock().unwrap();
        let last_user = new_users.last().unwrap();
        assert_eq!(output.unwrap(), "3".to_string());
        assert_eq!(*last_user, create_user(3));
    }

    #[test]
    fn test_adds_user_to_empty() {
        let users = Vec::new();
        let db = Arc::new(Mutex::new(users));
        let data = HashMap::from([
            ("name".to_string(), "test".to_string()),
            ("lastname".to_string(), "test1".to_string()),
        ]);
        let output = add_user(db.clone(), data, None);

        let new_users = db.lock().unwrap();
        let last_user = new_users.last().unwrap();
        assert_eq!(output.unwrap(), "0".to_string());
        assert_eq!(*last_user, create_user(0));
    }

    #[test]
    fn test_change_user_name() {
        let (_, db) = create_db();

        let change_data = HashMap::from([("name".to_string(), "test".to_string())]);
        let _ = change_user_data(db.clone(), 1, change_data);

        let new_users = db.lock().unwrap();
        let last_user = new_users.get(0).unwrap();
        assert_eq!(
            *last_user,
            User {
                id: 1,
                name: "test".to_string(),
                lastname: "Shutov".to_string()
            }
        );
    }

    #[test]
    fn test_change_user_lastname() {
        let (_, db) = create_db();
        let change_data = HashMap::from([("lastname".to_string(), "test1".to_string())]);
        let _ = change_user_data(db.clone(), 1, change_data);

        let new_users = db.lock().unwrap();
        let user = new_users.get(0).unwrap();
        assert_eq!(
            *user,
            User {
                id: 1,
                name: "Hlib".to_string(),
                lastname: "test1".to_string()
            }
        );
    }

    #[test]
    fn test_returns_error_if_not_exists() {
        let (_, db) = create_db();
        let change_data = HashMap::from([("lastname".to_string(), "test1".to_string())]);
        let output = change_user_data(db.clone(), 5, change_data);

        assert_eq!(output, Err(Errors::UserError(400)));
    }

    #[test]
    fn test_modify_user() {
        let (_, db) = create_db();
        let data = HashMap::from([
            ("name".to_string(), "test".to_string()),
            ("lastname".to_string(), "test1".to_string()),
        ]);
        let output = add_or_modify_user(db.clone(), 1, data);

        let new_users = db.lock().unwrap();
        let user = new_users.get(0).unwrap();

        assert_eq!(*user, create_user(1));
        assert_eq!(output, Ok("Modified user".to_string()));
    }

    #[test]
    fn test_creates_new_user() {
        let (_, db) = create_db();
        let data = HashMap::from([
            ("name".to_string(), "test".to_string()),
            ("lastname".to_string(), "test1".to_string()),
        ]);
        let output = add_or_modify_user(db.clone(), 3, data);

        let new_users = db.lock().unwrap();
        let user = new_users.last().unwrap();

        assert_eq!(*user, create_user(3));
        assert_eq!(output, Ok("Created new user".to_string()));
    }

    #[test]
    fn test_add_or_modify_user_returns_error_when_incorrect_data() {
        let (_, db) = create_db();
        let data = HashMap::from([
            ("test".to_string(), "test".to_string()),
            ("lastname".to_string(), "test1".to_string()),
        ]);
        let output = add_or_modify_user(db.clone(), 3, data);
        assert_eq!(output, Err(Errors::UserError(400)));
    }
}
