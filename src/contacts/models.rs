use crate::contacts::forms::ContactForm;
use serde::{Deserialize, Serialize};
use sqlx::postgres::Postgres;
use sqlx::Pool;
use std::vec::Vec;

#[derive(Debug, Deserialize, Serialize)]
pub struct Contact {
    pub id: i32,
    pub first: String,
    pub last: Option<String>,
    pub phone: String,
    pub email: String,
}

impl Contact {
    pub async fn count(pool: &Pool<Postgres>) -> u64 {
        let res = sqlx::query!("SELECT count(id) from contacts")
            .fetch_one(pool)
            .await
            .unwrap();
        res.count.unwrap().try_into().unwrap()
    }

    pub async fn all(pool: &Pool<Postgres>, offset: i64) -> Result<Vec<Contact>, sqlx::Error> {
        let contacts =
            sqlx::query_as!(Contact, "SELECT * FROM contacts OFFSET $1 LIMIT 5;", offset)
                .fetch_all(pool)
                .await;
        contacts
    }

    pub async fn get(pool: &Pool<Postgres>, id: i32) -> Result<Contact, sqlx::Error> {
        let contacts = sqlx::query_as!(
            Contact,
            "
                SELECT * FROM contacts
                WHERE
                    id = $1
            ",
            id
        )
        .fetch_one(pool)
        .await;
        contacts
    }

    pub async fn filter_email(
        pool: &Pool<Postgres>,
        value: &String,
    ) -> Result<Vec<Contact>, sqlx::Error> {
        let query = sqlx::query_as!(Contact, "SELECT * FROM contacts WHERE email = $1", value);
        query.fetch_all(pool).await
    }

    pub async fn search(pool: &Pool<Postgres>, query: String) -> Result<Vec<Contact>, sqlx::Error> {
        let contacts = sqlx::query_as!(
            Contact,
            "
                SELECT * FROM contacts
                WHERE
                    position($1 in first) > 0 OR
                    position($1 in last) > 0 OR
                    position($1 in phone) > 0 OR
                    position($1 in email) > 0
            ",
            query
        )
        .fetch_all(pool)
        .await;
        contacts
    }

    pub async fn create(pool: &Pool<Postgres>, form: ContactForm) -> Result<Contact, sqlx::Error> {
        sqlx::query_as!(
            Contact,
            r#"
                INSERT INTO contacts(first, last, phone, email)
                VALUES($1, $2, $3, $4)
                RETURNING *
            "#,
            form.first_name,
            form.last_name,
            form.phone,
            form.email
        )
        .fetch_one(pool)
        .await
    }

    pub async fn update(self, pool: &Pool<Postgres>) -> Result<Contact, sqlx::Error> {
        sqlx::query_as!(
            Contact,
            r#"
                UPDATE contacts
                SET
                    first = $1,
                    last = $2,
                    phone = $3,
                    email = $4
                WHERE id = $5
                RETURNING *
            "#,
            self.first,
            self.last,
            self.phone,
            self.email,
            self.id
        )
        .fetch_one(pool)
        .await
    }

    pub async fn delete(self, pool: &Pool<Postgres>) -> bool {
        let result = sqlx::query!(
            r#"
                DELETE FROM contacts
                WHERE id = $1
            "#,
            self.id
        )
        .execute(pool)
        .await;

        match result {
            Err(_) => false,
            Ok(r) => r.rows_affected() == 1,
        }
    }
}
