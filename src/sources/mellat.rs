use crate::sources::utils::de_currency;
use crate::sources::{Currency, SourceSingleUrlTrait};
use serde::Deserialize;

const API_URL: &str = "https://api.mellatbank.am/api/v1/rate/list";

#[derive(Debug, Deserialize)]
pub struct Response {
    pub status: u16,
    pub message: Vec<String>,
    #[serde(default)]
    pub result: Option<Result>,
}

impl SourceSingleUrlTrait for Response {
    fn url() -> String {
        API_URL.into()
    }
}

#[derive(Debug, Deserialize)]
pub struct Data {
    // _id: String,
    pub buy: f64,
    pub sell: f64,
    #[serde(deserialize_with = "de_currency")]
    pub currency: Currency,
    pub created_at: String,
    pub updated_at: String,
    pub updated: String,
    pub created: String,
    // __v: u8,
    #[serde(rename = "buyCash")]
    pub buy_cash: f64,
    #[serde(rename = "sellCash")]
    pub sell_cash: f64,
}

#[derive(Debug, Deserialize)]
pub struct Result {
    pub data: Vec<Data>,
}
