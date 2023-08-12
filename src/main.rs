use axum::body::Body;
use axum::extract::State;
use axum::http::{method::Method, Request, StatusCode};
use axum::response::{Html, Redirect};
use axum::routing::{get, post};
use axum::{Form, Json, Router};
use serde::{Deserialize, Serialize};
use std::vec::Vec;
use tera::{Context, Tera};

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let tera = match Tera::new("templates/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };
    let shared_tera = Arc::new(tera);

    let app = Router::new()
        .route("/", get(|| async { Redirect::permanent("/contacts") }))
        .route("/contacts", get(contacts))
        .route("/contacts/create", get(create_contact).post(create_contact))
        .with_state(shared_tera);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn contacts(State(state): State<Arc<Tera>>) -> Html<String> {
    let contacts = Contact::all().await;
    let mut context = Context::new();
    context.insert("contacts", &contacts);
    Html(state.render("contacts/list.html", &context).unwrap())
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
    State(tera): State<Arc<Tera>>,
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
            id: "".to_string(),
            first: "".to_string(),
            last: "".to_string(),
            phone: "".to_string(),
            email: "".to_string(),
        },
    );
    Html(tera.render("contacts/create.html", &context).unwrap())
}

#[derive(Deserialize, Serialize)]
struct Contact {
    id: String,
    first: String,
    last: String,
    phone: String,
    email: String,
}

impl Contact {
    async fn all() -> Vec<Contact> {
        vec![
            Contact {
                id: "aoeustheuoeuhoeu".to_string(),
                first: "Test".to_string(),
                last: "User".to_string(),
                phone: "1234567899".to_string(),
                email: "test@test.com".to_string(),
            },
            Contact {
                id: "oauaoeuoaeuoeauaoeu".to_string(),
                first: "Test2".to_string(),
                last: "User".to_string(),
                phone: "1234767899".to_string(),
                email: "test2@test.com".to_string(),
            },
        ]
    }

    fn search(query: String) -> Self {
        Contact {
            id: "aoeustheuoeuhoeu".to_string(),
            first: "Test".to_string(),
            last: "User".to_string(),
            phone: "1234567899".to_string(),
            email: "test@test.com".to_string(),
        }
    }
}
