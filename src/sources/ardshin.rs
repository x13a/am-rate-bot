use crate::sources::{de_currency, Currency as ModCurrency, JsonResponse};
use rust_decimal::Decimal;
use serde::Deserialize;

pub const API_URL: &str = "https://website-api.ardshinbank.am/currency";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub data: Data,
    pub updated_at: String,
}

impl JsonResponse for Response {
    fn url() -> String {
        API_URL.into()
    }
}

#[derive(Debug, Deserialize)]
pub struct Data {
    pub currencies: Currencies,
    pub gold: Gold,
}

#[derive(Debug, Deserialize)]
pub struct Currencies {
    pub cash: Vec<Currency>,
    pub no_cash: Vec<Currency>,
}

#[derive(Debug, Deserialize)]
pub struct Gold {
    pub cash: Vec<GoldCash>,
    pub no_cash: Vec<Currency>,
}

#[derive(Debug, Deserialize)]
pub struct Currency {
    #[serde(rename = "type", deserialize_with = "de_currency")]
    pub curr_type: ModCurrency,
    pub buy: Decimal,
    pub sell: Decimal,
    pub cb: String,
}

#[derive(Debug, Deserialize)]
pub struct GoldCash {
    pub quantity: String,
    pub rate: String,
}
