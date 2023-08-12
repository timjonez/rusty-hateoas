use axum::body::Body;
use axum::extract::State;
use axum::http::{method::Method, Request, StatusCode};
use axum::response::{Html, Redirect};
use axum::routing::{get, post};
use axum::{Form, Json, Router};
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
        .route("/contacts/create", get(create_contact).post(create_contact))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn contacts(State(app): State<Arc<AppState>>) -> Html<String> {
    let contacts = Contact::all(&app.db).await.unwrap();
    let mut context = Context::new();
    context.insert("contacts", &contacts);
    Html(app.tera.render("contacts/list.html", &context).unwrap())
}

#[derive(Debug, Deserialize, Serialize)]
struct ContactForm {
    first: String,
    last: String,
    email: String,
    phone: String,
}

async fn create_contact(
    method: Method,
    State(app): State<Arc<AppState>>,
    Form(form): Form<ContactForm>,
) -> Html<String> {
    match method {
        Method::GET => {
            println!("000000000000000")
        }
        Method::POST => {
            println!("111111111111111, {:?} {}", form, method)
        }
        _ => {
            println!("******************")
        }
    }
    let mut context = Context::new();
    context.insert(
        "contact",
        &Contact {
            id: 1,
            first: "".to_string(),
            last: Some("".to_string()),
            phone: "".to_string(),
            email: "".to_string(),
        },
    );
    Html(app.tera.render("contacts/create.html", &context).unwrap())
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

    fn search(query: String) -> Self {
        Contact {
            id: 1,
            first: "Test".to_string(),
            last: Some("User".to_string()),
            phone: "1234567899".to_string(),
            email: "test@test.com".to_string(),
        }
    }
}
