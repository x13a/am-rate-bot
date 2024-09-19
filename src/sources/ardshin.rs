use crate::sources::utils::{de_currency, de_f64};
use crate::sources::{Currency as SourcesCurrency, SourceSingleUrlTrait};
use serde::Deserialize;

const API_URL: &str = "https://website-api.ardshinbank.am/currency";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub data: Data,
    pub updated_at: String,
}

impl SourceSingleUrlTrait for Response {
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
    pub curr_type: SourcesCurrency,
    #[serde(deserialize_with = "de_f64")]
    pub buy: f64,
    #[serde(deserialize_with = "de_f64")]
    pub sell: f64,
    #[serde(deserialize_with = "de_f64")]
    pub cb: f64,
}

#[derive(Debug, Deserialize)]
pub struct GoldCash {
    pub quantity: String,
    #[serde(deserialize_with = "de_f64")]
    pub rate: f64,
}
