use crate::sources::utils::de_currency;
use crate::sources::{Currency as SourcesCurrency, SourceSingleUrlTrait};
use rust_decimal::serde::arbitrary_precision;
use rust_decimal::Decimal;
use serde::Deserialize;

pub const API_URL: &str = "https://api-user.vamprivet.ru/backend/api/v2/currencies/rates";

#[derive(Debug, Deserialize)]
pub struct Response {
    pub content: Vec<Content>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    pub id: String,
    pub timestamp: String,
    pub cpdt: String,
    pub publication_date: String,
    pub version: i32,
    pub benchmark_currency_id: String,
    pub buysell_timestamp: String,
    pub currency: Currency,
    pub value_base: Option<f64>,
    #[serde(deserialize_with = "arbitrary_precision::deserialize")]
    pub value_buy: Decimal,
    #[serde(deserialize_with = "arbitrary_precision::deserialize")]
    pub value_sell: Decimal,
    pub value_middle: f64,
    pub created: String,
    pub updated: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Currency {
    pub id: String,
    pub last_publication_date: String,
    pub last_cpd: String,
    #[serde(deserialize_with = "de_currency")]
    pub strcode: SourcesCurrency,
    pub name: String,
    pub country: String,
    pub global_id: u64,
    pub active_status: bool,
    pub created: String,
    pub updated: Option<String>,
}

impl SourceSingleUrlTrait for Response {
    fn url() -> String {
        API_URL.into()
    }
}
