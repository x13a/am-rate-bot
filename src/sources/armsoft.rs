use crate::sources::utils::de_currency;
use crate::sources::Currency;
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
    pub purchase: f64,
    pub rate_for: u16,
    pub sale: f64,
}
