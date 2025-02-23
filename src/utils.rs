use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::db_object::DataBase;
use crate::{db_object::UserEnum, User, UserGroup};

#[derive(Debug, PartialEq)]
pub enum Errors {
    ServerError(u16),
    UserError(u16),
}

pub struct UserController {
    database: Arc<Mutex<DataBase>>,
}

impl UserController {
    pub fn new(database: Arc<Mutex<DataBase>>) -> Self {
        Self { database }
    }
    pub fn show_users(&self) -> Result<String, Errors> {
        let users = self.database.lock().unwrap();
        let json = serde_json::to_string(&*users.get_all()).map_err(|_| Errors::ServerError(500));
        json
    }

    pub fn show_user(&self, id: u32) -> Result<String, Errors> {
        let users = self.database.lock().map_err(|_| Errors::ServerError(500))?;
        let user = users.get_one(id)?;
        let json = serde_json::to_string(user).map_err(|_| Errors::ServerError(500));
        json
    }

    pub fn add_user(
        &self,
        data: HashMap<String, String>,
        new_id: Option<u32>,
    ) -> Result<String, Errors> {
        let mut users = self.database.lock().map_err(|_| Errors::ServerError(500))?;
        if data.contains_key("name")
            && data.contains_key("lastname")
            && data.contains_key("birth_year")
            && data.contains_key("group")
        {
            let group = match data.get("group").unwrap().as_str() {
                "user" => UserGroup::User,
                "premium" => UserGroup::Premium,
                "admin" => UserGroup::Admin,
                _ => return Err(Errors::UserError(400)),
            };
            let user = User {
                id: 0,
                name: data.get("name").unwrap().to_owned(),
                lastname: data.get("lastname").unwrap().to_owned(),
                birth_year: data
                    .get("birth_year")
                    .unwrap()
                    .parse()
                    .map_err(|_| Errors::UserError(400))?,
                group,
            };
            let id = users.add_entry(user, new_id);

            Ok(format!("{}", id))
        } else {
            Err(Errors::UserError(400))
        }
    }

    pub fn change_user_data(
        &self,
        id: u32,
        change_data: HashMap<String, String>,
    ) -> Result<String, Errors> {
        let mut users = self.database.lock().map_err(|_| Errors::ServerError(500))?;

        let mut change_data_enums = Vec::new();
        for (key, value) in change_data {
            let data_enum = match key.as_str() {
                "name" => UserEnum::Name(value.to_owned()),
                "lastname" => UserEnum::Lastname(value.to_owned()),
                "birth_year" => {
                    UserEnum::BirthYear(value.parse().map_err(|_| Errors::UserError(400))?)
                }
                "group" => {
                    let group = match value.as_str() {
                        "user" => UserGroup::User,
                        "premium" => UserGroup::Premium,
                        "admin" => UserGroup::Admin,
                        _ => return Err(Errors::UserError(400)),
                    };
                    UserEnum::Group(group)
                }
                _ => {
                    return Err(Errors::UserError(400));
                }
            };
            change_data_enums.push(data_enum);
        }
        users.change_user(id, change_data_enums)?;
        Ok("Changed".to_string())
    }

    pub fn delete_user(&self, id: u32) -> Result<String, Errors> {
        let mut users = self.database.lock().map_err(|_| Errors::ServerError(500))?;
        users.remove_entry(id)?;
        Ok("Removed user".to_string())
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
            birth_year: 2000,
            group: crate::UserGroup::Premium,
        }
    }
    fn create_db() -> (Vec<User>, Arc<Mutex<DataBase>>) {
        let user_1 = User {
            id: 1,
            name: "Hlib".to_string(),
            lastname: "Shutov".to_string(),
            birth_year: 2000,
            group: crate::UserGroup::Admin,
        };
        let user_2 = User {
            id: 2,
            name: "Wojciech".to_string(),
            lastname: "Oczkowski".to_string(),
            birth_year: 2000,
            group: crate::UserGroup::User,
        };
        let users = vec![user_1, user_2];
        let db = Arc::new(Mutex::new(DataBase { db: users.clone() }));

        (users, db)
    }
    fn create_controller(db: Arc<Mutex<DataBase>>) -> UserController {
        UserController::new(db)
    }
    #[test]
    fn test_show_users() {
        let (users, db) = create_db();
        let controller = create_controller(db);
        let output = controller.show_users();
        let result: Vec<User> = serde_json::from_str(output.unwrap().as_str()).unwrap();
        assert_eq!(result, users);
    }

    #[test]
    fn test_show_user() {
        let (users, db) = create_db();
        let controller = create_controller(db);
        let output = controller.show_user(1);
        let result: User = serde_json::from_str(output.unwrap().as_str()).unwrap();
        assert_eq!(result, users[0]);
    }

    #[test]
    fn test_adds_user_to_the_end() {
        let (_, db) = create_db();
        let controller = create_controller(db.clone());
        let data = HashMap::from([
            ("name".to_string(), "test".to_string()),
            ("lastname".to_string(), "test1".to_string()),
            ("birth_year".to_string(), "2000".to_string()),
            ("group".to_string(), "premium".to_string()),
        ]);
        let output = controller.add_user(data, None);

        let new_users = db.lock().unwrap();
        let last_user = new_users.db.last().unwrap().clone();
        assert_eq!(output.unwrap(), "3".to_string());
        assert_eq!(last_user, create_user(3));
    }

    #[test]
    fn test_change_user_name() {
        let (_, db) = create_db();
        let controller = create_controller(db.clone());

        let change_data = HashMap::from([("name".to_string(), "test".to_string())]);
        let result = controller.change_user_data(1, change_data);

        let new_users = db.lock().unwrap();
        let user = new_users.db.get(0).unwrap();

        assert_eq!(result, Ok("Changed".to_string()));
        assert_eq!(
            *user,
            User {
                id: 1,
                name: "test".to_string(),
                lastname: "Shutov".to_string(),
                birth_year: 2000,
                group: crate::UserGroup::Admin,
            }
        );
    }

    #[test]
    fn test_change_user_lastname() {
        let (_, db) = create_db();
        let controller = create_controller(db.clone());

        let change_data = HashMap::from([
            ("group".to_string(), "premium".to_string()),
            ("birth_year".to_string(), "2009".to_string()),
        ]);
        let result = controller.change_user_data(1, change_data);

        let new_users = db.lock().unwrap();
        let user = new_users.db.get(0).unwrap();

        assert_eq!(result, Ok("Changed".to_string()));
        assert_eq!(
            *user,
            User {
                id: 1,
                name: "Hlib".to_string(),
                lastname: "Shutov".to_string(),
                birth_year: 2009,
                group: crate::UserGroup::Premium,
            }
        );
    }

    #[test]
    fn test_returns_error_if_not_exists() {
        let (_, db) = create_db();
        let controller = create_controller(db);
        let change_data = HashMap::from([("lastname".to_string(), "test1".to_string())]);
        let output = controller.change_user_data(5, change_data);

        assert_eq!(output, Err(Errors::UserError(400)));
    }

    #[test]
    fn test_delete_user() {
        let (_, db) = create_db();
        let controller = create_controller(db.clone());
        let result = controller.delete_user(2);

        let users = &db.lock().unwrap().db;

        assert_eq!(result, Ok("Removed user".to_string()));
        assert_eq!(
            *users,
            vec!(User {
                id: 1,
                name: "Hlib".to_string(),
                lastname: "Shutov".to_string(),
                birth_year: 2000,
                group: crate::UserGroup::Admin,
            })
        );
    }
}
