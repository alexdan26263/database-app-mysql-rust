use axum::http::StatusCode;
use sqlx::{mysql::{MySqlConnectOptions, MySqlPoolOptions}, MySqlPool};
use std::env;

pub type ApiError = (StatusCode, &'static str);
pub type ApiResult<T> = Result<T, ApiError>;

pub async fn connect() -> Result<MySqlPool, Box<dyn std::error::Error>> {

    dotenvy::dotenv().ok();

    let options = MySqlConnectOptions::new()
        .host(&env::var("DB_HOST")?)
        .port(env::var("DB_PORT")?.parse()?)
        .database(&env::var("DB_NAME")?)
        .username(&env::var("DB_USER")?)
        .password(&env::var("DB_PASSWORD")?);

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;
    Ok(pool)
}

pub fn sql_error(error: sqlx::Error) -> ApiError {

    eprintln!("Eroare MySQL: {error}");
    if let sqlx::Error::Database(ref db) = error {
        match db.code().as_deref() {
            Some("1062") => return (StatusCode::CONFLICT, "Inregistrarea exista deja"),
            Some("1452") => return (StatusCode::BAD_REQUEST, "Cheie straina inexistenta"),
            Some("3819") => return (StatusCode::BAD_REQUEST, "Conditie SQL incalcata"),
            Some("1406") => return (StatusCode::BAD_REQUEST, "Text prea lung"),
            _ => {}
        }
    }
    (StatusCode::INTERNAL_SERVER_ERROR, "Eroare baza de date")
}

pub fn missing() -> ApiError {
    (StatusCode::NOT_FOUND, "Inregistrarea nu exista")
}

pub fn valid_text(value: &str, limit: usize) -> bool {
    !value.trim().is_empty() && value.chars().count() <= limit
}
