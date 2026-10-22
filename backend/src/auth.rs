//! Autentificare cu parole Argon2id si sesiuni Bearer stocate ca hash SHA-256.
//! Sesiunile expira dupa 12 ore. Nu trimitem hash-ul parolei sau al tokenului clientului.

use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, MySqlPool};

use crate::db::{self, ApiResult};

#[derive(Debug, Deserialize)]
struct LoginInput {
    username: String,
    password: String,
}

#[derive(Debug, FromRow)]
struct DbUser {
    utilizator_id: u32,
    username: String,
    password_hash: String,
    rol: String,
}

#[derive(Debug, FromRow)]
struct SessionUser {
    utilizator_id: u32,
    username: String,
    rol: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CurrentUser {
    pub utilizator_id: u32,
    pub username: String,
    pub rol: String,
    #[serde(skip_serializing)]
    pub token_hash: String,
}

impl CurrentUser {
    pub fn require_admin(&self) -> ApiResult<()> {
        if self.rol == "administrator" {
            Ok(())
        } else {
            Err((StatusCode::FORBIDDEN, "Doar administratorul poate sterge"))
        }
    }
}

#[derive(Serialize)]
struct LoginResponse {
    token: String,
    token_type: &'static str,
    expires_in_seconds: u64,
    utilizator_id: u32,
    username: String,
    rol: String,
}

pub fn public_router() -> Router<MySqlPool> {
    Router::new().route("/api/auth/login", post(login))
}

pub fn private_router() -> Router<MySqlPool> {
    Router::new()
        .route("/api/auth/me", get(me))
        .route("/api/auth/logout", post(logout))
}

fn sha256_hex(input: &str) -> String {
    hex::encode(Sha256::digest(input.as_bytes()))
}

async fn login(
    State(pool): State<MySqlPool>,
    Json(input): Json<LoginInput>,
) -> ApiResult<Json<LoginResponse>> {
    if input.username.trim().is_empty()
        || input.username.chars().count() > 80
        || input.password.len() > 1024
    {
        return Err((StatusCode::BAD_REQUEST, "Date de autentificare invalide"));
    }

    let user = sqlx::query_as::<_, DbUser>(
        "SELECT UtilizatorID AS utilizator_id, Username AS username, \
                PasswordHash AS password_hash, Rol AS rol \
         FROM Utilizatori WHERE Username=?",
    )
    .bind(&input.username)
    .fetch_optional(&pool)
    .await
    .map_err(db::sql_error)?;

    let user = user.ok_or((StatusCode::UNAUTHORIZED, "Username sau parola incorecta"))?;

    let hash = user.password_hash.clone();
    let password = input.password;
    let valid = tokio::task::spawn_blocking(move || {
        PasswordHash::new(&hash)
            .map(|parsed| {
                Argon2::default()
                    .verify_password(password.as_bytes(), &parsed)
                    .is_ok()
            })
            .unwrap_or(false)
    })
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Eroare autentificare"))?;

    if !valid {
        return Err((StatusCode::UNAUTHORIZED, "Username sau parola incorecta"));
    }

    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Eroare generare token"))?;
    let token = hex::encode(bytes);
    let token_hash = sha256_hex(&token);

    sqlx::query(
        "INSERT INTO Sesiuni (TokenHash, UtilizatorID, ExpiraLa) \
         VALUES (?, ?, DATE_ADD(UTC_TIMESTAMP(), INTERVAL 12 HOUR))",
    )
    .bind(token_hash)
    .bind(user.utilizator_id)
    .execute(&pool)
    .await
    .map_err(db::sql_error)?;

    Ok(Json(LoginResponse {
        token,
        token_type: "Bearer",
        expires_in_seconds: 12 * 3600,
        utilizator_id: user.utilizator_id,
        username: user.username,
        rol: user.rol,
    }))
}

async fn me(Extension(user): Extension<CurrentUser>) -> Json<CurrentUser> {
    Json(user)
}

async fn logout(
    State(pool): State<MySqlPool>,
    Extension(user): Extension<CurrentUser>,
) -> ApiResult<StatusCode> {
    sqlx::query("DELETE FROM Sesiuni WHERE TokenHash=?")
        .bind(&user.token_hash)
        .execute(&pool)
        .await
        .map_err(db::sql_error)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn require_login(
    State(pool): State<MySqlPool>,
    mut request: Request,
    next: Next,
) -> Response {
    let raw_token = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));

    let Some(raw_token) = raw_token else {
        return (StatusCode::UNAUTHORIZED, "Token Bearer lipsa").into_response();
    };
    if raw_token.len() != 64 || !raw_token.bytes().all(|b| b.is_ascii_hexdigit()) {
        return (StatusCode::UNAUTHORIZED, "Token invalid").into_response();
    }

    let token_hash = sha256_hex(raw_token);
    let session = sqlx::query_as::<_, SessionUser>(
        "SELECT u.UtilizatorID AS utilizator_id, \
                u.Username AS username, u.Rol AS rol \
         FROM Sesiuni s JOIN Utilizatori u ON u.UtilizatorID=s.UtilizatorID \
         WHERE s.TokenHash=? AND s.ExpiraLa > UTC_TIMESTAMP()",
    )
    .bind(&token_hash)
    .fetch_optional(&pool)
    .await;

    match session {
        Ok(Some(user)) => {
            request.extensions_mut().insert(CurrentUser {
                utilizator_id: user.utilizator_id,
                username: user.username,
                rol: user.rol,
                token_hash,
            });
            next.run(request).await
        }
        Ok(None) => (StatusCode::UNAUTHORIZED, "Token invalid sau expirat").into_response(),
        Err(error) => {
            eprintln!("Eroare validare sesiune: {error}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Eroare baza de date").into_response()
        }
    }
}
