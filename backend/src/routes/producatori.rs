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
    models::{Producator, ProducatorInput},
};

const SELECT_LIST: &str = "SELECT ProducatorID AS producator_id, Denumire AS denumire, \
     TaraOrigine AS tara_origine, Adresa AS adresa FROM Producatori ORDER BY ProducatorID";
const SELECT_ONE: &str = "SELECT ProducatorID AS producator_id, Denumire AS denumire, \
     TaraOrigine AS tara_origine, Adresa AS adresa FROM Producatori WHERE ProducatorID = ?";

pub fn router() -> Router<MySqlPool> {
    Router::new()
        .route("/api/producatori", get(list).post(create))
        .route("/api/producatori/{id}", get(one).put(update).delete(remove))
}

fn valid(input: &ProducatorInput) -> bool {
    db::valid_text(&input.denumire, 150)
        && db::valid_text(&input.tara_origine, 100)
        && db::valid_text(&input.adresa, 255)
}

async fn fetch(pool: &MySqlPool, id: u32) -> ApiResult<Option<Producator>> {
    sqlx::query_as::<_, Producator>(SELECT_ONE)
        .bind(id).fetch_optional(pool).await.map_err(db::sql_error)
}

async fn list(State(pool): State<MySqlPool>) -> ApiResult<Json<Vec<Producator>>> {
    let rows = sqlx::query_as::<_, Producator>(SELECT_LIST)
        .fetch_all(&pool).await.map_err(db::sql_error)?;
    Ok(Json(rows))
}

async fn one(
    State(pool): State<MySqlPool>,
    Path(id): Path<u32>,
) -> ApiResult<Json<Producator>> {
    Ok(Json(fetch(&pool, id).await?.ok_or_else(db::missing)?))
}

async fn create(
    State(pool): State<MySqlPool>,
    Json(input): Json<ProducatorInput>,
) -> ApiResult<(StatusCode, Json<Producator>)> {
    if !valid(&input) {
        return Err((StatusCode::BAD_REQUEST, "Date producator invalide"));
    }
    let result = sqlx::query(
        "INSERT INTO Producatori (Denumire, TaraOrigine, Adresa) VALUES (?, ?, ?)"
    )
        .bind(input.denumire.trim())
        .bind(input.tara_origine.trim())
        .bind(input.adresa.trim())
        .execute(&pool).await.map_err(db::sql_error)?;
    let id = result.last_insert_id() as u32;
    Ok((StatusCode::CREATED, Json(fetch(&pool, id).await?.ok_or_else(db::missing)?)))
}

async fn update(
    State(pool): State<MySqlPool>,
    Path(id): Path<u32>,
    Json(input): Json<ProducatorInput>,
) -> ApiResult<Json<Producator>> {
    if !valid(&input) {
        return Err((StatusCode::BAD_REQUEST, "Date producator invalide"));
    }
    sqlx::query(
        "UPDATE Producatori SET Denumire=?, TaraOrigine=?, Adresa=? WHERE ProducatorID=?"
    )
        .bind(input.denumire.trim())
        .bind(input.tara_origine.trim())
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
    let result = sqlx::query("DELETE FROM Producatori WHERE ProducatorID=?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(db::sql_error)?;
    if result.rows_affected() == 0 {
        return Err(db::missing());
    }
    Ok(StatusCode::NO_CONTENT)
}
