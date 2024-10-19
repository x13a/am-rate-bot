pub use crate::sources::RatesConfig as Config;
use crate::sources::{de, Currency as ModCurrency};
use rust_decimal::serde::arbitrary_precision;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Response {
    pub content: Vec<Content>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    pub currency: Currency,
    #[serde(deserialize_with = "arbitrary_precision::deserialize")]
    pub value_buy: Decimal,
    #[serde(deserialize_with = "arbitrary_precision::deserialize")]
    pub value_sell: Decimal,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Currency {
    #[serde(deserialize_with = "de::currency")]
    pub strcode: ModCurrency,
}
