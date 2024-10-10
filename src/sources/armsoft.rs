use crate::sources::{de_currency, Currency};
use rust_decimal::serde::arbitrary_precision;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase", transparent)]
pub struct Response {
    pub array_of_exchange_rate: Vec<ExchangeRate>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ExchangeRate {
    #[serde(rename = "CBRate")]
    pub cb_rate: f64,
    #[serde(deserialize_with = "de_currency")]
    pub currency: Currency,
    #[serde(deserialize_with = "arbitrary_precision::deserialize")]
    pub purchase: Decimal,
    pub rate_for: u32,
    #[serde(deserialize_with = "arbitrary_precision::deserialize")]
    pub sale: Decimal,
}
