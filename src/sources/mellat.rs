pub use crate::sources::SourceConfig as Config;
use crate::sources::{de, Currency, JsonResponse};
use rust_decimal::serde::arbitrary_precision;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Response {
    pub result: Result,
}

#[derive(Debug, Deserialize)]
pub struct Data {
    #[serde(deserialize_with = "arbitrary_precision::deserialize")]
    pub buy: Decimal,
    #[serde(deserialize_with = "arbitrary_precision::deserialize")]
    pub sell: Decimal,
    #[serde(deserialize_with = "de::currency")]
    pub currency: Currency,
    #[serde(
        rename = "buyCash",
        deserialize_with = "arbitrary_precision::deserialize"
    )]
    pub buy_cash: Decimal,
    #[serde(
        rename = "sellCash",
        deserialize_with = "arbitrary_precision::deserialize"
    )]
    pub sell_cash: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct Result {
    pub data: Vec<Data>,
}

impl JsonResponse for Response {}
