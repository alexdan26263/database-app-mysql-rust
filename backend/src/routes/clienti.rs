use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Extension, Json, Router,
};
use sqlx::MySqlPool;
use crate::{
    auth::CurrentUser,
    db::{self, ApiResult},
    models::{Client, ClientInput},
};

const SELECT_LIST: &str = "SELECT ClientID AS client_id, Nume AS nume, Prenume AS prenume, \
     Adresa AS adresa FROM Clienti ORDER BY ClientID";
const SELECT_ONE: &str = "SELECT ClientID AS client_id, Nume AS nume, Prenume AS prenume, \
     Adresa AS adresa FROM Clienti WHERE ClientID = ?";

pub fn router() -> Router<MySqlPool> {
    Router::new()
        .route("/api/clienti", get(list).post(create))
        .route("/api/clienti/{id}", get(one).put(update).delete(remove))
}

fn valid(input: &ClientInput) -> bool {
    db::valid_text(&input.nume, 100)
        && db::valid_text(&input.prenume, 100)
        && db::valid_text(&input.adresa, 255)
}

async fn fetch(pool: &MySqlPool, id: u32) -> ApiResult<Option<Client>> {
    sqlx::query_as::<_, Client>(SELECT_ONE)
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(db::sql_error)
}

async fn list(State(pool): State<MySqlPool>) -> ApiResult<Json<Vec<Client>>> {
    let rows = sqlx::query_as::<_, Client>(SELECT_LIST)
        .fetch_all(&pool).await.map_err(db::sql_error)?;
    Ok(Json(rows))
}

async fn one(
    State(pool): State<MySqlPool>,
    Path(id): Path<u32>,
) -> ApiResult<Json<Client>> {
    Ok(Json(fetch(&pool, id).await?.ok_or_else(db::missing)?))
}

async fn create(
    State(pool): State<MySqlPool>,
    Json(input): Json<ClientInput>,
) -> ApiResult<(StatusCode, Json<Client>)> {
    if !valid(&input) {
        return Err((StatusCode::BAD_REQUEST, "Nume, prenume sau adresa invalide"));
    }
    let result = sqlx::query(
        "INSERT INTO Clienti (Nume, Prenume, Adresa) VALUES (?, ?, ?)"
    )
        .bind(input.nume.trim())
        .bind(input.prenume.trim())
        .bind(input.adresa.trim())
        .execute(&pool).await.map_err(db::sql_error)?;
    let id = result.last_insert_id() as u32;
    let saved = fetch(&pool, id).await?.ok_or_else(db::missing)?;
    Ok((StatusCode::CREATED, Json(saved)))
}

async fn update(
    State(pool): State<MySqlPool>,
    Path(id): Path<u32>,
    Json(input): Json<ClientInput>,
) -> ApiResult<Json<Client>> {
    if !valid(&input) {
        return Err((StatusCode::BAD_REQUEST, "Nume, prenume sau adresa invalide"));
    }
    sqlx::query("UPDATE Clienti SET Nume=?, Prenume=?, Adresa=? WHERE ClientID=?")
        .bind(input.nume.trim())
        .bind(input.prenume.trim())
        .bind(input.adresa.trim())
        .bind(id)
        .execute(&pool).await.map_err(db::sql_error)?;
    Ok(Json(fetch(&pool, id).await?.ok_or_else(db::missing)?))
}

async fn remove(
    State(pool): State<MySqlPool>,
    Path(id): Path<u32>,
    Extension(user): Extension<CurrentUser>,
) -> ApiResult<StatusCode> {
    user.require_admin()?;
    let result = sqlx::query("DELETE FROM Clienti WHERE ClientID=?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(db::sql_error)?;
    if result.rows_affected() == 0 {
        return Err(db::missing());
    }
    Ok(StatusCode::NO_CONTENT)
}
