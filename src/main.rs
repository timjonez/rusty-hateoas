use axum::body::Body;
use axum::extract::{Json, Path, Query, RawForm, State};
use axum::http::{header::HeaderMap, method::Method, Request, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{delete, get, post};
use axum::{Form, Router};
use serde::{Deserialize, Serialize};
use sqlx::Pool;
use std::vec::Vec;
use tera::{Context, Tera};
use tower_http::services::ServeDir;

use sqlx::postgres::{PgPool, PgPoolOptions, Postgres};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

struct AppState {
    tera: Tera,
    db: Pool<Postgres>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let db_connection_str = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_connection_str)
        .await
        .expect("can't connect to database");

    let tera = match Tera::new("templates/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };
    let state = Arc::new(AppState { tera, db: pool });

    let app = Router::new()
        .nest_service("/static", ServeDir::new("static"))
        .route("/validate/email", get(validate_email))
        .route("/", get(|| async { Redirect::permanent("/contacts") }))
        .route("/contacts", get(contacts).delete(delete_contact_list))
        .route("/contacts/count", get(num_contacts))
        .route(
            "/contacts/:user_id",
            get(get_contact).delete(delete_contact),
        )
        .route(
            "/contacts/:user_id/edit",
            get(get_edit_contact).post(edit_contact),
        )
        .route(
            "/contacts/create",
            get(get_create_contact).post(create_contact),
        )
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn contacts(
    headers: HeaderMap,
    State(app): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Html<String> {
    let page: i32 = match params.contains_key("page") {
        true => params.get("page").unwrap().parse().unwrap(),
        false => 1,
    };
    let offset: i64 = ((page - 1) * 5).into();
    let contacts = match params.get("q") {
        Some(q) => Contact::search(&app.db, q.to_string()).await.unwrap(),
        None => Contact::all(&app.db, offset).await.unwrap(),
    };
    let mut context = Context::new();
    context.insert("page", &page);
    context.insert("contacts", &contacts);
    if headers.contains_key("hx-trigger") {
        if headers.get("hx-trigger").unwrap() == "search" {
            return Html(app.tera.render("contacts/_rows.html", &context).unwrap());
        }
    }
    Html(app.tera.render("contacts/list.html", &context).unwrap())
}

async fn num_contacts(State(app): State<Arc<AppState>>) -> String {
    let count = Contact::count(&app.db).await;
    format!("{} total contacts", count)
}

async fn get_contact(State(app): State<Arc<AppState>>, Path(user_id): Path<i32>) -> Html<String> {
    let mut context = Context::new();
    let contact = Contact::get(&app.db, user_id).await.unwrap();
    context.insert("contact", &contact);
    Html(app.tera.render("contacts/detail.html", &context).unwrap())
}

async fn validate_email(
    State(app): State<Arc<AppState>>,
    Query(args): Query<HashMap<String, String>>,
) -> String {
    let email = args.get("email").unwrap();
    let contacts = Contact::filter_email(&app.db, email).await.unwrap();
    match contacts.len() {
        0 => return String::new(),
        _ => return String::from("This email is already taken"),
    }
}

async fn get_edit_contact(
    State(app): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
) -> Html<String> {
    let mut context = Context::new();
    let contact = Contact::get(&app.db, user_id).await.unwrap();
    let form = ContactForm::from(&contact);
    context.insert("form", &form);
    context.insert("contact", &contact);
    context.insert("errors", &HashMap::<String, String>::new());
    Html(app.tera.render("contacts/edit.html", &context).unwrap())
}

async fn edit_contact(
    State(app): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
    Form(form): Form<ContactForm>,
) -> Response {
    let mut context = Context::new();
    let (valid, errors) = form.is_valid();
    if valid {
        let _ = form.update(user_id, &app.db).await;
        return Redirect::to("/contacts").into_response();
    }
    context.insert("errors", &errors);
    context.insert("form", &form);
    Html(app.tera.render("contacts/create.html", &context).unwrap()).into_response()
}

#[derive(Debug, Deserialize, Serialize)]
struct ContactForm {
    first_name: String,
    last_name: String,
    email: String,
    phone: String,
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
    fn new() -> Self {
        ContactForm {
            first_name: String::new(),
            last_name: String::new(),
            email: String::new(),
            phone: String::new(),
        }
    }
    fn is_valid(&self) -> (bool, HashMap<String, String>) {
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
    async fn save(self, pool: &Pool<Postgres>) -> Result<Contact, sqlx::Error> {
        Contact::create(&pool, self).await
    }
    async fn update(self, id: i32, pool: &Pool<Postgres>) -> Result<Contact, sqlx::Error> {
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

async fn get_create_contact(State(app): State<Arc<AppState>>) -> Html<String> {
    let mut context = Context::new();
    context.insert("form", &ContactForm::new());
    context.insert("errors", &HashMap::<String, String>::new());
    Html(app.tera.render("contacts/create.html", &context).unwrap())
}

async fn create_contact(
    State(app): State<Arc<AppState>>,
    Form(form): Form<ContactForm>,
) -> Response {
    let mut context = Context::new();
    let (valid, errors) = form.is_valid();
    if valid {
        let _ = form.save(&app.db).await;
        return Redirect::to("/contacts").into_response();
    }
    context.insert("errors", &errors);
    context.insert("form", &form);
    Html(app.tera.render("contacts/create.html", &context).unwrap()).into_response()
}

async fn delete_contact(
    headers: HeaderMap,
    State(app): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
) -> Response {
    let success = match Contact::get(&app.db, user_id).await {
        Err(e) => {
            println!("could not delete contact {}: {}", user_id, e);
            false
        }
        Ok(contact) => {
            let _ = contact.delete(&app.db).await;
            true
        }
    };
    let is_delete_btn = match headers.get("hx-trigger") {
        None => false,
        Some(id) => id == "delete-btn",
    };

    if is_delete_btn {
        return Redirect::to("/contacts").into_response();
    }
    "".into_response()
}

async fn delete_contact_list(State(app): State<Arc<AppState>>, raw_form: RawForm) -> Response {
    let mut ids: Vec<i32> = Vec::new();
    let str_form = match std::str::from_utf8(&raw_form.0) {
        Ok(s) => s,
        Err(e) => panic!("{}", e),
    };

    for i in str_form.split("&") {
        let parts: Vec<&str> = i.split("=").collect();
        if parts.len() == 2 && parts.first().unwrap() == &"selected_contact_ids" {
            let str_id = parts.last().unwrap();
            match str_id.parse::<i32>() {
                Ok(id) => ids.push(id.into()),
                Err(_) => {}
            }
        }
    }

    for id in ids {
        match Contact::get(&app.db, id).await {
            Err(_) => {}
            Ok(contact) => {
                let _ = contact.delete(&app.db).await;
            }
        }
    }
    return Redirect::to("/contacts").into_response();
}

#[derive(Debug, Deserialize, Serialize)]
struct Contact {
    id: i32,
    first: String,
    last: Option<String>,
    phone: String,
    email: String,
}

impl Contact {
    async fn count(pool: &Pool<Postgres>) -> u64 {
        let res = sqlx::query!("SELECT count(id) from contacts")
            .fetch_one(pool)
            .await
            .unwrap();
        res.count.unwrap().try_into().unwrap()
    }

    async fn all(pool: &Pool<Postgres>, offset: i64) -> Result<Vec<Contact>, sqlx::Error> {
        let contacts =
            sqlx::query_as!(Contact, "SELECT * FROM contacts OFFSET $1 LIMIT 5;", offset)
                .fetch_all(pool)
                .await;
        contacts
    }

    async fn get(pool: &Pool<Postgres>, id: i32) -> Result<Contact, sqlx::Error> {
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

    async fn filter_email(
        pool: &Pool<Postgres>,
        value: &String,
    ) -> Result<Vec<Contact>, sqlx::Error> {
        let query = sqlx::query_as!(Contact, "SELECT * FROM contacts WHERE email = $1", value);
        query.fetch_all(pool).await
    }

    async fn search(pool: &Pool<Postgres>, query: String) -> Result<Vec<Contact>, sqlx::Error> {
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

    async fn create(pool: &Pool<Postgres>, form: ContactForm) -> Result<Contact, sqlx::Error> {
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

    async fn update(self, pool: &Pool<Postgres>) -> Result<Contact, sqlx::Error> {
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

    async fn delete(self, pool: &Pool<Postgres>) -> bool {
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
