use std::{collections::HashMap, sync::Arc, sync::Mutex};

pub fn show_users(db: Arc<Mutex<Vec<HashMap<&str, String>>>>) -> String{
    let users = db.lock().unwrap();
    println!("{:?}", users);
    let json = serde_json::to_string(&*users).unwrap();
    json
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_show_users() {
        let user_1 = HashMap::from([
            ("id", "1".to_string()),
            ("name", "Hlib".to_string()),
            ("lastname", "Shutov".to_string()),
        ]);
        let user_2 = HashMap::from([
            ("id", "2".to_string()),
            ("name", "Wojciech".to_string()),
            ("lastname", "Oczkowski".to_string()),
        ]);
        let users = vec![user_1, user_2];
        let db = Arc::new(Mutex::new(users.clone()));
        let output = show_users(db);
        let result: Vec<HashMap<&str, String>> = serde_json::from_str(output.as_str()).unwrap();
        assert_eq!(result, users);
    }
}
