pub use crate::sources::SourceConfig as Config;
use crate::sources::{de, Currency, RateTypeJsonResponse};
use rust_decimal::serde::arbitrary_precision;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Response {
    pub rates: Vec<Rate>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Rate {
    #[serde(deserialize_with = "arbitrary_precision::deserialize")]
    pub buy: Decimal,
    #[serde(deserialize_with = "de::currency")]
    pub id: Currency,
    #[serde(deserialize_with = "arbitrary_precision::deserialize")]
    pub sale: Decimal,
    #[serde(deserialize_with = "arbitrary_precision::deserialize")]
    pub unit: Decimal,
}

impl RateTypeJsonResponse for Response {}
