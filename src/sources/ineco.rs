pub use crate::sources::RatesConfig as Config;
use crate::sources::{de, Currency};
use rust_decimal::serde::arbitrary_precision_option;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Response {
    pub items: Vec<Item>,
}

#[derive(Debug, Deserialize)]
pub struct Item {
    #[serde(deserialize_with = "de::currency")]
    pub code: Currency,
    pub cash: Rate,
    pub cashless: Rate,
}

#[derive(Debug, Deserialize)]
pub struct Rate {
    #[serde(deserialize_with = "arbitrary_precision_option::deserialize")]
    pub buy: Option<Decimal>,
    #[serde(deserialize_with = "arbitrary_precision_option::deserialize")]
    pub sell: Option<Decimal>,
}
