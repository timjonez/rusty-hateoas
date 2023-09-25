use sqlx::postgres::Postgres;
use sqlx::Pool;
use tera::Tera;

pub struct AppState {
    pub tera: Tera,
    pub db: Pool<Postgres>,
}
