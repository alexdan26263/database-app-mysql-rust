use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct Client {
    pub client_id: u32,
    pub nume: String,
    pub prenume: String,
    pub adresa: String,
}

#[derive(Debug, Deserialize)]
pub struct ClientInput {
    pub nume: String,
    pub prenume: String,
    pub adresa: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Produs {
    pub produs_id: u32,
    pub denumire: String,
    pub data_producere: NaiveDate,
    pub data_expirare: NaiveDate,
}

#[derive(Debug, Deserialize)]
pub struct ProdusInput {
    pub denumire: String,
    pub data_producere: NaiveDate,
    pub data_expirare: NaiveDate,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Producator {
    pub producator_id: u32,
    pub denumire: String,
    pub tara_origine: String,
    pub adresa: String,
}

#[derive(Debug, Deserialize)]
pub struct ProducatorInput {
    pub denumire: String,
    pub tara_origine: String,
    pub adresa: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ClientProdus {
    pub client_id: u32,
    pub produs_id: u32,
    pub nume: String,
    pub prenume: String,
    pub adresa_client: String,
    pub denumire_produs: String,
    pub data_producere: NaiveDate,
    pub data_expirare: NaiveDate,
}

#[derive(Debug, Deserialize)]
pub struct ClientProdusInput {
    pub client_id: u32,
    pub produs_id: u32,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ProdusProducator {
    pub produs_id: u32,
    pub producator_id: u32,
    pub denumire_produs: String,
    pub data_producere: NaiveDate,
    pub data_expirare: NaiveDate,
    pub denumire_producator: String,
    pub tara_origine: String,
    pub adresa_producator: String,
}

#[derive(Debug, Deserialize)]
pub struct ProdusProducatorInput {
    pub produs_id: u32,
    pub producator_id: u32,
}
