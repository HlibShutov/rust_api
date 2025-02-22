use crate::{Errors, User, UserGroup};

#[derive(Clone, Debug, PartialEq)]
pub struct DataBase {
    pub db: Vec<User>,
}

pub enum UserEnum {
    Name(String),
    Lastname(String),
    BirthYear(u16),
    Group(UserGroup),
}
impl DataBase {
    pub fn new() -> Self {
        Self { db: Vec::new() }
    }
    pub fn add_entry(&mut self, mut user: User, new_id: Option<u32>) -> u32 {
        let last_user = self.db.last();
        let id = new_id.unwrap_or(if let Some(last_user) = last_user {
            last_user.id + 1
        } else {
            0
        });
        user.id = id;

        self.db.push(user);

        id
    }
    pub fn remove_entry(&mut self, id: u32) -> Result<usize, Errors> {
        let user = self
            .db
            .iter()
            .position(|user| user.id == id)
            .ok_or(Errors::UserError(400))?;
        self.db.remove(user);
        Ok(user)
    }
    pub fn change_user(&mut self, id: u32, data: Vec<UserEnum>) -> Result<usize, Errors> {
        let user_id = self
            .db
            .iter()
            .position(|user| user.id == id)
            .ok_or(Errors::UserError(400))?;

        let user = self.db.get_mut(user_id).unwrap();

        data.iter().for_each(|change_data| match change_data {
            UserEnum::Name(name) => user.name = name.to_owned(),
            UserEnum::Lastname(lastname) => user.lastname = lastname.to_owned(),
            UserEnum::BirthYear(birth_year) => user.birth_year = *birth_year,
            UserEnum::Group(group) => user.group = group.clone(),
        });

        Ok(user_id)
    }

    pub fn get_all(&self) -> &Vec<User> {
        &self.db
    }

    pub fn get_one(&self, id: u32) -> Result<&User, Errors> {
        let user_id = self
            .db
            .iter()
            .position(|user| user.id == id)
            .ok_or(Errors::UserError(400))?;
        Ok(&self.db.get(user_id).unwrap())
    }
}
