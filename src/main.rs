use axum::response::Redirect;
use axum::routing::{delete, get, post};
use axum::Router;
use tera::Tera;
use tower_http::services::ServeDir;

use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use rusty_hateoas::app::AppState;
use rusty_hateoas::contacts::routes as contact_routes;

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

    let tera = match Tera::new("templates/**/*.*ml") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };
    let state = Arc::new(AppState { tera, db: pool });

    let app = Router::new()
        .nest_service("/static", ServeDir::new("static"))
        .route("/validate/email", get(contact_routes::validate_email))
        .route("/", get(|| async { Redirect::permanent("/contacts") }))
        .route("/app/", get(contact_routes::app_contacts))
        .route(
            "/contacts",
            get(contact_routes::contacts).delete(contact_routes::delete_contact_list),
        )
        .route("/contacts/count", get(contact_routes::num_contacts))
        .route(
            "/contacts/:user_id",
            get(contact_routes::get_contact).delete(contact_routes::delete_contact),
        )
        .route(
            "/contacts/:user_id/edit",
            get(contact_routes::get_edit_contact).post(contact_routes::edit_contact),
        )
        .route(
            "/contacts/create",
            get(contact_routes::get_create_contact).post(contact_routes::create_contact),
        )
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
