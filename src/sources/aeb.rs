pub use crate::sources::RatesConfig as Config;
use crate::sources::{de, Currency, RateType};
use rust_decimal::serde::arbitrary_precision_option;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    #[serde(deserialize_with = "de::currency")]
    pub main_currency_code: Currency,
    pub rate_currency_settings: Vec<Item>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    #[serde(deserialize_with = "de::currency")]
    pub currency_code: Currency,
    pub rates: Vec<Rate>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rate {
    #[serde(deserialize_with = "arbitrary_precision_option::deserialize")]
    pub buy_rate: Option<Decimal>,
    #[serde(deserialize_with = "arbitrary_precision_option::deserialize")]
    pub sell_rate: Option<Decimal>,
    #[serde(rename = "type", deserialize_with = "de::rate_type")]
    pub rate_type: RateType,
}
