use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, Redirect};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::vec::Vec;
use tera::{Context, Tera};

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
        .route("/contacts/create", get(create_contact))
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

async fn create_contact(State(state): State<Arc<Tera>>) -> Html<String> {
    let context = Context::new();
    Html(state.render("contacts/create.html", &context).unwrap())
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
