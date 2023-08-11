use axum::http::StatusCode;
use axum::response::Html;
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::vec::Vec;
use tera::{Context, Tera};

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let app = Router::new()
        .route("/", get(index))
        .route("/contacts", get(contacts));

    axum::Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index() -> Html<String> {
    let tera = match Tera::new("templates/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };
    Html(tera.render("index.html", &Context::new()).unwrap())
}

async fn contacts() -> Json<Vec<Contact>> {
    let contacts = Contact::all().await;
    Json(contacts)
}

#[derive(Deserialize, Serialize)]
struct Contact {
    first: String,
    last: String,
    phone: String,
    email: String,
}

impl Contact {
    async fn all() -> Vec<Contact> {
        vec![Contact {
            first: "Test".to_string(),
            last: "User".to_string(),
            phone: "1234567899".to_string(),
            email: "test@test.com".to_string(),
        }]
    }

    fn search(query: String) -> Self {
        Contact {
            first: "Test".to_string(),
            last: "User".to_string(),
            phone: "1234567899".to_string(),
            email: "test@test.com".to_string(),
        }
    }
}
