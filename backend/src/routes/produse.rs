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
    models::{Produs, ProdusInput},
};

const SELECT_LIST: &str = "SELECT ProdusID AS produs_id, Denumire AS denumire, \
     DataProducere AS data_producere, DataExpirare AS data_expirare \
     FROM ProdusAlimentar ORDER BY ProdusID";
const SELECT_ONE: &str = "SELECT ProdusID AS produs_id, Denumire AS denumire, \
     DataProducere AS data_producere, DataExpirare AS data_expirare \
     FROM ProdusAlimentar WHERE ProdusID = ?";

pub fn router() -> Router<MySqlPool> {
    Router::new()
        .route("/api/produse", get(list).post(create))
        .route("/api/produse/{id}", get(one).put(update).delete(remove))
}

fn valid(input: &ProdusInput) -> bool {
    db::valid_text(&input.denumire, 150)
        && input.data_expirare >= input.data_producere
}

async fn fetch(pool: &MySqlPool, id: u32) -> ApiResult<Option<Produs>> {
    sqlx::query_as::<_, Produs>(SELECT_ONE)
        .bind(id).fetch_optional(pool).await.map_err(db::sql_error)
}

async fn list(State(pool): State<MySqlPool>) -> ApiResult<Json<Vec<Produs>>> {
    let rows = sqlx::query_as::<_, Produs>(SELECT_LIST)
        .fetch_all(&pool).await.map_err(db::sql_error)?;
    Ok(Json(rows))
}

async fn one(
    State(pool): State<MySqlPool>,
    Path(id): Path<u32>,
) -> ApiResult<Json<Produs>> {
    Ok(Json(fetch(&pool, id).await?.ok_or_else(db::missing)?))
}

async fn create(
    State(pool): State<MySqlPool>,
    Json(input): Json<ProdusInput>,
) -> ApiResult<(StatusCode, Json<Produs>)> {
    if !valid(&input) {
        return Err((StatusCode::BAD_REQUEST, "Denumire sau date invalide"));
    }
    let result = sqlx::query(
        "INSERT INTO ProdusAlimentar (Denumire, DataProducere, DataExpirare) \
         VALUES (?, ?, ?)"
    )
        .bind(input.denumire.trim())
        .bind(input.data_producere)
        .bind(input.data_expirare)
        .execute(&pool).await.map_err(db::sql_error)?;
    let id = result.last_insert_id() as u32;
    Ok((StatusCode::CREATED, Json(fetch(&pool, id).await?.ok_or_else(db::missing)?)))
}

async fn update(
    State(pool): State<MySqlPool>,
    Path(id): Path<u32>,
    Json(input): Json<ProdusInput>,
) -> ApiResult<Json<Produs>> {
    if !valid(&input) {
        return Err((StatusCode::BAD_REQUEST, "Denumire sau date invalide"));
    }
    sqlx::query(
        "UPDATE ProdusAlimentar SET Denumire=?, DataProducere=?, \
         DataExpirare=? WHERE ProdusID=?"
    )
        .bind(input.denumire.trim())
        .bind(input.data_producere)
        .bind(input.data_expirare)
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
    let result = sqlx::query("DELETE FROM ProdusAlimentar WHERE ProdusID=?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(db::sql_error)?;
    if result.rows_affected() == 0 {
        return Err(db::missing());
    }
    Ok(StatusCode::NO_CONTENT)
}
