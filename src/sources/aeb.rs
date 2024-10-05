use crate::sources::utils::{de_currency, de_rate_type};
use crate::sources::{Currency, RateType, SourceSingleUrlTrait};
use rust_decimal::serde::arbitrary_precision_option;
use rust_decimal::Decimal;
use serde::Deserialize;

pub const API_URL: &str = "https://mobile.aeb.am/mobile-proxy-exchange-rates/rate-settings";

impl SourceSingleUrlTrait for Response {
    fn url() -> String {
        API_URL.into()
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    #[serde(deserialize_with = "de_currency")]
    pub main_currency_code: Currency,
    pub rate_currency_settings: Vec<Item>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    #[serde(deserialize_with = "de_currency")]
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
    #[serde(rename = "type", deserialize_with = "de_rate_type")]
    pub rate_type: RateType,
}
