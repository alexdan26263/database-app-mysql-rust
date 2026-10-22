use argon2::{password_hash::{PasswordHasher, SaltString}, Argon2};
use sqlx::{mysql::MySqlConnectOptions, MySqlPool};
use std::{env, error::Error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 || !["client", "administrator"].contains(&args[2].as_str()) {
        eprintln!("Utilizare: cargo run --bin create_user -- <username> <client|administrator>");
        std::process::exit(2);
    }
    let username = args[1].trim();
    if username.is_empty() || username.chars().count() > 80 {
        return Err("Username invalid (1-80 caractere)".into());
    }

    let password = rpassword::prompt_password("Parola noua (minim 12 caractere): ")?;
    let confirm = rpassword::prompt_password("Repeta parola: ")?;
    if password != confirm || password.chars().count() < 12 {
        return Err("Parolele difera sau au sub 12 caractere".into());
    }

    let mut salt_bytes = [0u8; 16];
    getrandom::fill(&mut salt_bytes)
        .map_err(|_| "Nu am putut genera un salt aleator")?;
    let salt = SaltString::encode_b64(&salt_bytes)
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    // Hash PHC ($argon2id$...) compatibil cu verificarea din auth.rs.
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| std::io::Error::other(e.to_string()))?
        .to_string();

    let options = MySqlConnectOptions::new()
        .host(&env::var("DB_HOST")?)
        .port(env::var("DB_PORT")?.parse()?)
        .database(&env::var("DB_NAME")?)
        .username(&env::var("DB_USER")?)
        .password(&env::var("DB_PASSWORD")?);
    let pool = MySqlPool::connect_with(options).await?;

    sqlx::query("INSERT INTO Utilizatori (Username, PasswordHash, Rol) VALUES (?, ?, ?)")
        .bind(username)
        .bind(hash)
        .bind(args[2].as_str())
        .execute(&pool)
        .await?;
    println!("Cont creat: {username} ({})", args[2]);
    Ok(())
}
