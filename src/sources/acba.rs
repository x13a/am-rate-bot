pub use crate::sources::SourceConfig as Config;
use crate::sources::{de, Currency, JsonResponse};
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Response {
    pub result: Result,
}

#[derive(Debug, Deserialize)]
pub struct Result {
    pub rates: Rates,
}

#[derive(Debug, Deserialize)]
pub struct Rates {
    pub cash: Vec<Rate>,
    pub non_cash: Vec<Rate>,
    pub cross: Vec<CrossRate>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Rate {
    pub buy: Decimal,
    pub sell: Decimal,
    #[serde(deserialize_with = "de::currency")]
    pub currency: Currency,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CrossRate {
    pub buy: Decimal,
    pub sell: Decimal,
    pub currency: String,
}

impl JsonResponse for Response {}
