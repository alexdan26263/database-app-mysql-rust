
use axum::{
    extract::State,
    http::StatusCode,
    routing::get,
    Json, Router,
};
use chrono::NaiveDate;
use serde::Serialize;
use sqlx::{
    mysql::{MySqlConnectOptions, MySqlPoolOptions},
    FromRow, MySqlPool,
};
use std::env;

#[derive(Serialize, FromRow)]
struct Client {
    client_id: u32,
    nume: String,
    prenume: String,
    adresa: String,
}

#[derive(Serialize, FromRow)]
struct Produs {
    produs_id: u32,
    denumire: String,
    data_producere: NaiveDate,
    data_expirare: NaiveDate,
}

#[derive(Serialize, FromRow)]
struct Producator {
    producator_id: u32,
    denumire: String,
    tara_origine: String,
    adresa: String,
}

#[derive(Serialize, FromRow)]
struct ClientProdus {
    client_id: u32,
    produs_id: u32,
    nume: String,
    prenume: String,
    adresa_client: String,
    denumire_produs: String,
}

#[derive(Serialize, FromRow)]
struct ProdusProducator {
    produs_id: u32,
    producator_id: u32,
    denumire_produs: String,
    denumire_producator: String,
    tara_origine: String,
}

type ApiError = (StatusCode, &'static str);

async fn health() -> &'static str {
    "Server Rust Axum functional!"
}

async fn get_clienti(
    State(pool): State<MySqlPool>,
) -> Result<Json<Vec<Client>>, ApiError> {
    let rows = sqlx::query_as::<_, Client>(
        "SELECT ClientID AS client_id,
                Nume AS nume,
                Prenume AS prenume,
                Adresa AS adresa
         FROM Clienti ORDER BY ClientID"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        eprintln!("Eroare MySQL: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Eroare baza de date")
    })?;

    Ok(Json(rows))
}

async fn get_produse(
    State(pool): State<MySqlPool>,
) -> Result<Json<Vec<Produs>>, ApiError> {
    let rows = sqlx::query_as::<_, Produs>(
        "SELECT ProdusID AS produs_id,
                Denumire AS denumire,
                DataProducere AS data_producere,
                DataExpirare AS data_expirare
         FROM ProdusAlimentar ORDER BY ProdusID"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        eprintln!("Eroare MySQL: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Eroare baza de date")
    })?;

    Ok(Json(rows))
}

async fn get_producatori(
    State(pool): State<MySqlPool>,
) -> Result<Json<Vec<Producator>>, ApiError> {
    let rows = sqlx::query_as::<_, Producator>(
        "SELECT ProducatorID AS producator_id,
                Denumire AS denumire,
                TaraOrigine AS tara_origine,
                Adresa AS adresa
         FROM Producatori ORDER BY ProducatorID"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        eprintln!("Eroare MySQL: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Eroare baza de date")
    })?;

    Ok(Json(rows))
}

async fn get_clienti_produse(
    State(pool): State<MySqlPool>,
) -> Result<Json<Vec<ClientProdus>>, ApiError> {
    let rows = sqlx::query_as::<_, ClientProdus>(
        "SELECT ClientID AS client_id,
                ProdusID AS produs_id,
                Nume AS nume,
                Prenume AS prenume,
                AdresaClient AS adresa_client,
                DenumireProdus AS denumire_produs
         FROM V_ClientiProduse"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        eprintln!("Eroare MySQL: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Eroare baza de date")
    })?;

    Ok(Json(rows))
}

async fn get_produse_producatori(
    State(pool): State<MySqlPool>,
) -> Result<Json<Vec<ProdusProducator>>, ApiError> {
    let rows = sqlx::query_as::<_, ProdusProducator>(
        "SELECT ProdusID AS produs_id,
                ProducatorID AS producator_id,
                DenumireProdus AS denumire_produs,
                DenumireProducator AS denumire_producator,
                TaraOrigine AS tara_origine
         FROM V_ProduseProducatori"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        eprintln!("Eroare MySQL: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Eroare baza de date")
    })?;

    Ok(Json(rows))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;

    let host = env::var("DB_HOST")?;
    let port: u16 = env::var("DB_PORT")?.parse()?;
    let database = env::var("DB_NAME")?;
    let username = env::var("DB_USER")?;
    let password = env::var("DB_PASSWORD")?;

    let options = MySqlConnectOptions::new()
        .host(&host)
        .port(port)
        .database(&database)
        .username(&username)
        .password(&password);

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    println!("Conectare reusita");

    let app = Router::new()
        .route("/", get(health))
        .route("/api/clienti", get(get_clienti))
        .route("/api/produse", get(get_produse))
        .route("/api/producatori", get(get_producatori))
        .route("/api/clienti-produse", get(get_clienti_produse))
        .route("/api/produse-producatori", get(get_produse_producatori))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind(
        "127.0.0.1:3000"
    ).await?;

    println!("Server live http://localhost:3000");

    axum::serve(listener, app).await?;

    Ok(())
}
