pub use crate::sources::SourceConfig as Config;
use crate::sources::{de, Currency, JsonResponse};
use rust_decimal::serde::arbitrary_precision;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Response {
    pub items: Vec<ResponseItem>,
}

#[derive(Debug, Deserialize)]
pub struct ResponseItem {
    #[serde(rename = "categoryId")]
    pub category_id: String,
    pub rates: Rates,
}

#[derive(Debug, Deserialize)]
pub struct Rates {
    pub items: Vec<RatesItem>,
}

#[derive(Debug, Deserialize)]
pub struct RatesItem {
    pub base: BaseTarget,
    #[serde(default)]
    pub buy: Option<BuySell>,
    #[serde(default)]
    pub sell: Option<BuySell>,
    pub target: BaseTarget,
}

#[derive(Debug, Deserialize)]
pub struct BaseTarget {
    #[serde(deserialize_with = "de::currency")]
    pub currency: Currency,
}

#[derive(Debug, Deserialize)]
pub struct BuySell {
    #[serde(deserialize_with = "arbitrary_precision::deserialize")]
    pub min: Decimal,
}

impl JsonResponse for Response {}
