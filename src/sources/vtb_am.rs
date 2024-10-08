use crate::sources::{de_currency, Currency, JsonResponse};
use rust_decimal::serde::arbitrary_precision;
use rust_decimal::Decimal;
use serde::Deserialize;

pub const API_URL: &str = "https://online.vtb.am/dbo/api/v1/currencies/rates";

impl JsonResponse for Response {
    fn url() -> String {
        API_URL.into()
    }
}

#[derive(Debug, Deserialize)]
pub struct Response {
    pub count: usize,
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
    pub count: usize,
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
    pub timestamp: String,
}

#[derive(Debug, Deserialize)]
pub struct BaseTarget {
    #[serde(deserialize_with = "de_currency")]
    pub currency: Currency,
    pub discriminator: u32,
}

#[derive(Debug, Deserialize)]
pub struct BuySell {
    pub close: f64,
    #[serde(deserialize_with = "arbitrary_precision::deserialize")]
    pub max: Decimal,
    #[serde(deserialize_with = "arbitrary_precision::deserialize")]
    pub min: Decimal,
    pub open: f64,
    pub trend: String,
}
