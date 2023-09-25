use crate::contacts::models::Contact;
use serde::{Deserialize, Serialize};
use sqlx::postgres::Postgres;
use sqlx::Pool;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct ContactForm {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
}

impl From<&Contact> for ContactForm {
    fn from(contact: &Contact) -> Self {
        let last_name = match contact.last.clone() {
            Some(n) => n,
            None => String::new(),
        };
        Self {
            first_name: contact.first.clone(),
            last_name: last_name,
            email: contact.email.clone(),
            phone: contact.phone.clone(),
        }
    }
}

impl ContactForm {
    pub fn new() -> Self {
        ContactForm {
            first_name: String::new(),
            last_name: String::new(),
            email: String::new(),
            phone: String::new(),
        }
    }
    pub fn is_valid(&self) -> (bool, HashMap<String, String>) {
        let mut errs = HashMap::new();
        if !self.first_name.contains("test") {
            errs.insert(
                "first_name".to_string(),
                "First name must contain \"Test\"".to_string(),
            );
            return (false, errs);
        }
        (true, errs)
    }
    pub async fn save(self, pool: &Pool<Postgres>) -> Result<Contact, sqlx::Error> {
        Contact::create(&pool, self).await
    }
    pub async fn update(self, id: i32, pool: &Pool<Postgres>) -> Result<Contact, sqlx::Error> {
        let contact = Contact {
            id,
            first: self.first_name,
            last: Some(self.last_name),
            phone: self.phone,
            email: self.email,
        };
        Contact::update(contact, &pool).await
    }
}
