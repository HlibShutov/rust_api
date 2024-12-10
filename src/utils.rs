use std::sync::{Arc, Mutex};

use crate::User;

pub fn show_users(db: Arc<Mutex<Vec<User>>>) -> String {
    let users = db.lock().unwrap();
    let json = serde_json::to_string(&*users).unwrap();
    json
}

pub fn show_user(db: Arc<Mutex<Vec<User>>>, id: u32) -> String {
    let users = db.lock().unwrap();
    let user = users.iter().find(|user| user.id == id).unwrap();
    let json = serde_json::to_string(user).unwrap();
    json
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let db = Arc::new(Mutex::new(users.clone()));
        let output = show_users(db);
        let result: Vec<User> = serde_json::from_str(output.as_str()).unwrap();
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
        let db = Arc::new(Mutex::new(users.clone()));
        let output = show_user(db, 1);
        let result: User = serde_json::from_str(output.as_str()).unwrap();
        assert_eq!(result, user_1);
    }
}
