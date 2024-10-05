use crate::sources::utils::de_currency;
use crate::sources::{Currency, SourceSingleUrlTrait};
use rust_decimal::serde::arbitrary_precision;
use rust_decimal::Decimal;
use serde::Deserialize;

pub const API_URL: &str = "https://api.mellatbank.am/api/v1/rate/list";

#[derive(Debug, Deserialize)]
pub struct Response {
    pub status: i32,
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
    #[serde(deserialize_with = "arbitrary_precision::deserialize")]
    pub buy: Decimal,
    #[serde(deserialize_with = "arbitrary_precision::deserialize")]
    pub sell: Decimal,
    #[serde(deserialize_with = "de_currency")]
    pub currency: Currency,
    pub created_at: String,
    pub updated_at: String,
    pub updated: String,
    pub created: String,
    // __v: i32,
    #[serde(
        rename = "buyCash",
        deserialize_with = "arbitrary_precision::deserialize"
    )]
    pub buy_cash: Decimal,
    #[serde(
        rename = "sellCash",
        deserialize_with = "arbitrary_precision::deserialize"
    )]
    pub sell_cash: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct Result {
    pub data: Vec<Data>,
}
