pub use crate::sources::RatesConfig as Config;
use crate::sources::{de, Currency, RatesConfigTrait};
use reqwest::Client;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Response {
    pub result: Result,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Result {
    pub currency_rate: Vec<CurrencyRate>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CurrencyRate {
    #[serde(deserialize_with = "de::empty_decimal")]
    pub buy: Option<Decimal>,
    pub cards_buy: String,
    pub cards_sell: String,
    #[serde(deserialize_with = "de::empty_decimal")]
    pub csh_buy: Option<Decimal>,
    #[serde(deserialize_with = "de::empty_decimal")]
    pub csh_buy_trf: Option<Decimal>,
    #[serde(deserialize_with = "de::empty_decimal")]
    pub csh_sell: Option<Decimal>,
    #[serde(deserialize_with = "de::empty_decimal")]
    pub csh_sell_trf: Option<Decimal>,
    #[serde(deserialize_with = "de::currency")]
    pub iso_txt: Currency,
    #[serde(deserialize_with = "de::empty_decimal")]
    pub sell: Option<Decimal>,
}

pub async fn get<T>(client: &Client, config: &T) -> anyhow::Result<Response>
where
    T: RatesConfigTrait,
{
    let resp = client
        .post(config.rates_url())
        .header(reqwest::header::CONTENT_LENGTH, 0)
        .send()
        .await?
        .json()
        .await?;
    Ok(resp)
}
