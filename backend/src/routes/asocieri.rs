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
    models::{ClientProdus, ClientProdusInput, ProdusProducator, ProdusProducatorInput},
};

const CP_LIST: &str = "SELECT ClientID AS client_id, ProdusID AS produs_id, \
     Nume AS nume, Prenume AS prenume, AdresaClient AS adresa_client, \
     DenumireProdus AS denumire_produs, DataProducere AS data_producere, \
     DataExpirare AS data_expirare FROM V_ClientiProduse ORDER BY ClientID, ProdusID";
const CP_ONE: &str = "SELECT ClientID AS client_id, ProdusID AS produs_id, \
     Nume AS nume, Prenume AS prenume, AdresaClient AS adresa_client, \
     DenumireProdus AS denumire_produs, DataProducere AS data_producere, \
     DataExpirare AS data_expirare FROM V_ClientiProduse WHERE ClientID=? AND ProdusID=?";

const PP_LIST: &str = "SELECT ProdusID AS produs_id, ProducatorID AS producator_id, \
     DenumireProdus AS denumire_produs, DataProducere AS data_producere, \
     DataExpirare AS data_expirare, DenumireProducator AS denumire_producator, \
     TaraOrigine AS tara_origine, AdresaProducator AS adresa_producator \
     FROM V_ProduseProducatori ORDER BY ProdusID, ProducatorID";
const PP_ONE: &str = "SELECT ProdusID AS produs_id, ProducatorID AS producator_id, \
     DenumireProdus AS denumire_produs, DataProducere AS data_producere, \
     DataExpirare AS data_expirare, DenumireProducator AS denumire_producator, \
     TaraOrigine AS tara_origine, AdresaProducator AS adresa_producator \
     FROM V_ProduseProducatori WHERE ProdusID=? AND ProducatorID=?";

pub fn router() -> Router<MySqlPool> {
    Router::new()
        .route("/api/clienti-produse", get(list_cp).post(create_cp))
        .route("/api/clienti-produse/{client_id}/{produs_id}", get(one_cp).put(update_cp).delete(remove_cp))
        .route("/api/produse-producatori", get(list_pp).post(create_pp))
        .route("/api/produse-producatori/{produs_id}/{producator_id}", get(one_pp).put(update_pp).delete(remove_pp))
}

async fn fetch_cp(pool: &MySqlPool, client: u32, produs: u32) -> ApiResult<Option<ClientProdus>> {
    sqlx::query_as::<_, ClientProdus>(CP_ONE)
        .bind(client).bind(produs)
        .fetch_optional(pool).await.map_err(db::sql_error)
}

async fn fetch_pp(pool: &MySqlPool, produs: u32, producator: u32) -> ApiResult<Option<ProdusProducator>> {
    sqlx::query_as::<_, ProdusProducator>(PP_ONE)
        .bind(produs).bind(producator)
        .fetch_optional(pool).await.map_err(db::sql_error)
}

async fn list_cp(State(pool): State<MySqlPool>) -> ApiResult<Json<Vec<ClientProdus>>> {
    Ok(Json(sqlx::query_as::<_, ClientProdus>(CP_LIST)
        .fetch_all(&pool).await.map_err(db::sql_error)?))
}

async fn one_cp(
    State(pool): State<MySqlPool>,
    Path((client, produs)): Path<(u32, u32)>,
) -> ApiResult<Json<ClientProdus>> {
    Ok(Json(fetch_cp(&pool, client, produs).await?.ok_or_else(db::missing)?))
}

async fn create_cp(
    State(pool): State<MySqlPool>,
    Json(input): Json<ClientProdusInput>,
) -> ApiResult<(StatusCode, Json<ClientProdus>)> {
    if input.client_id == 0 || input.produs_id == 0 {
        return Err((StatusCode::BAD_REQUEST, "ID-urile trebuie sa fie pozitive"));
    }
    sqlx::query("INSERT INTO ClientiProduse (ClientID, ProdusID) VALUES (?, ?)")
        .bind(input.client_id).bind(input.produs_id)
        .execute(&pool).await.map_err(db::sql_error)?;
    let row = fetch_cp(&pool, input.client_id, input.produs_id)
        .await?.ok_or_else(db::missing)?;
    Ok((StatusCode::CREATED, Json(row)))
}

async fn update_cp(
    State(pool): State<MySqlPool>,
    Path((old_client, old_produs)): Path<(u32, u32)>,
    Json(input): Json<ClientProdusInput>,
) -> ApiResult<Json<ClientProdus>> {
    if input.client_id == 0 || input.produs_id == 0 {
        return Err((StatusCode::BAD_REQUEST, "ID-urile trebuie sa fie pozitive"));
    }
    let mut tx = pool.begin().await.map_err(db::sql_error)?;
    let existing: Option<i32> = sqlx::query_scalar(
        "SELECT 1 FROM ClientiProduse WHERE ClientID=? AND ProdusID=? FOR UPDATE"
    )
        .bind(old_client).bind(old_produs)
        .fetch_optional(&mut *tx).await.map_err(db::sql_error)?;
    if existing.is_none() {
        return Err(db::missing());
    }
    sqlx::query(
        "UPDATE ClientiProduse SET ClientID=?, ProdusID=? WHERE ClientID=? AND ProdusID=?"
    )
        .bind(input.client_id).bind(input.produs_id)
        .bind(old_client).bind(old_produs)
        .execute(&mut *tx).await.map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(Json(fetch_cp(&pool, input.client_id, input.produs_id)
        .await?.ok_or_else(db::missing)?))
}

async fn list_pp(State(pool): State<MySqlPool>) -> ApiResult<Json<Vec<ProdusProducator>>> {
    Ok(Json(sqlx::query_as::<_, ProdusProducator>(PP_LIST)
        .fetch_all(&pool).await.map_err(db::sql_error)?))
}

async fn one_pp(
    State(pool): State<MySqlPool>,
    Path((produs, producator)): Path<(u32, u32)>,
) -> ApiResult<Json<ProdusProducator>> {
    Ok(Json(fetch_pp(&pool, produs, producator).await?.ok_or_else(db::missing)?))
}

async fn create_pp(
    State(pool): State<MySqlPool>,
    Json(input): Json<ProdusProducatorInput>,
) -> ApiResult<(StatusCode, Json<ProdusProducator>)> {
    if input.produs_id == 0 || input.producator_id == 0 {
        return Err((StatusCode::BAD_REQUEST, "ID-urile trebuie sa fie pozitive"));
    }
    sqlx::query("INSERT INTO ProduseProducatori (ProdusID, ProducatorID) VALUES (?, ?)")
        .bind(input.produs_id).bind(input.producator_id)
        .execute(&pool).await.map_err(db::sql_error)?;
    let row = fetch_pp(&pool, input.produs_id, input.producator_id)
        .await?.ok_or_else(db::missing)?;
    Ok((StatusCode::CREATED, Json(row)))
}

async fn update_pp(
    State(pool): State<MySqlPool>,
    Path((old_produs, old_producator)): Path<(u32, u32)>,
    Json(input): Json<ProdusProducatorInput>,
) -> ApiResult<Json<ProdusProducator>> {
    if input.produs_id == 0 || input.producator_id == 0 {
        return Err((StatusCode::BAD_REQUEST, "ID-urile trebuie sa fie pozitive"));
    }
    let mut tx = pool.begin().await.map_err(db::sql_error)?;
    let existing: Option<i32> = sqlx::query_scalar(
        "SELECT 1 FROM ProduseProducatori WHERE ProdusID=? AND ProducatorID=? FOR UPDATE"
    )
        .bind(old_produs).bind(old_producator)
        .fetch_optional(&mut *tx).await.map_err(db::sql_error)?;
    if existing.is_none() {
        return Err(db::missing());
    }
    sqlx::query(
        "UPDATE ProduseProducatori SET ProdusID=?, ProducatorID=? \
         WHERE ProdusID=? AND ProducatorID=?"
    )
        .bind(input.produs_id).bind(input.producator_id)
        .bind(old_produs).bind(old_producator)
        .execute(&mut *tx).await.map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(Json(fetch_pp(&pool, input.produs_id, input.producator_id)
        .await?.ok_or_else(db::missing)?))
}

async fn remove_cp(
    State(pool): State<MySqlPool>,
    Path((client_id, produs_id)): Path<(u32, u32)>,
    Extension(user): Extension<CurrentUser>,
) -> ApiResult<StatusCode> {
    user.require_admin()?;
    let result = sqlx::query(
        "DELETE FROM ClientiProduse WHERE ClientID=? AND ProdusID=?",
    )
        .bind(client_id).bind(produs_id)
        .execute(&pool).await.map_err(db::sql_error)?;
    if result.rows_affected() == 0 {
        return Err(db::missing());
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn remove_pp(
    State(pool): State<MySqlPool>,
    Path((produs_id, producator_id)): Path<(u32, u32)>,
    Extension(user): Extension<CurrentUser>,
) -> ApiResult<StatusCode> {
    user.require_admin()?;
    let result = sqlx::query(
        "DELETE FROM ProduseProducatori WHERE ProdusID=? AND ProducatorID=?",
    )
        .bind(produs_id).bind(producator_id)
        .execute(&pool).await.map_err(db::sql_error)?;
    if result.rows_affected() == 0 {
        return Err(db::missing());
    }
    Ok(StatusCode::NO_CONTENT)
}
