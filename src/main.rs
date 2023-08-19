use axum::body::Body;
use axum::extract::{Json, Query, RawForm, State, Path};
use axum::http::{method::Method, Request, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Form, Router};
use serde::{Deserialize, Serialize};
use sqlx::Pool;
use std::vec::Vec;
use tera::{Context, Tera};

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
        .route("/", get(|| async { Redirect::permanent("/contacts") }))
        .route("/contacts", get(contacts))
        .route("/contacts/:user_id", get(get_contact))
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
    State(app): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Html<String> {
    let contacts = match params.get("q") {
        Some(q) => Contact::search(&app.db, q.to_string()).await.unwrap(),
        None => Contact::all(&app.db).await.unwrap(),
    };
    let mut context = Context::new();
    context.insert("contacts", &contacts);
    Html(app.tera.render("contacts/list.html", &context).unwrap())
}

async fn get_contact(State(app): State<Arc<AppState>>, Path(user_id): Path<i32>) -> Html<String> {
    let mut context = Context::new();
    let contact = Contact::get(&app.db, user_id).await.unwrap();
    context.insert("contact", &contact);
    Html(app.tera.render("contacts/detail.html", &context).unwrap())
}

#[derive(Debug, Deserialize, Serialize)]
struct ContactForm {
    first_name: String,
    last_name: String,
    email: String,
    phone: String,
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
            errs.insert("first_name".to_string(), "First name must contain \"Test\"".to_string());
            return (false, errs)
        }
        (true, errs)
    }
    async fn save(self, pool: &Pool<Postgres>) -> Result<Contact, sqlx::Error> {
        Contact::create(&pool, self).await
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

#[derive(Debug, Deserialize, Serialize)]
struct Contact {
    id: i32,
    first: String,
    last: Option<String>,
    phone: String,
    email: String,
}

impl Contact {
    async fn all(pool: &Pool<Postgres>) -> Result<Vec<Contact>, sqlx::Error> {
        let contacts = sqlx::query_as!(Contact, "SELECT * FROM contacts;")
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
}
