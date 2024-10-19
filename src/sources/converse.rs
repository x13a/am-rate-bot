pub use crate::sources::RatesConfig as Config;
use crate::sources::{de as de_utils, Currency as ModCurrency};
use rust_decimal::serde::arbitrary_precision;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Response {
    #[serde(rename = "Non Cash")]
    pub non_cash: Vec<Item>,
    #[serde(rename = "Cash")]
    pub cash: Vec<Item>,
}

#[derive(Debug, Deserialize)]
pub struct Item {
    #[serde(deserialize_with = "arbitrary_precision::deserialize")]
    pub buy: Decimal,
    #[serde(deserialize_with = "arbitrary_precision::deserialize")]
    pub sell: Decimal,
    #[serde(deserialize_with = "de_utils::currency")]
    pub iso2: ModCurrency,
    pub currency: Currency,
}

#[derive(Debug, Deserialize)]
pub struct Currency {
    #[serde(deserialize_with = "de_utils::currency")]
    pub iso: ModCurrency,
    pub use_for_rates: i32,
}
