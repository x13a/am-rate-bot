use crate::sources::Error;
use rust_decimal::Decimal;
use select::{document::Document, predicate::Attr};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub rates_url: String,
    pub enabled: bool,
    pub commission_rate: Decimal,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub initial_items: Vec<Item>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub currencies_data: Vec<CurrencyData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyData {
    pub value: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Value {
    pub exchange_rate: Vec<ValueRate>,
    #[serde(default)]
    pub conversion_rate: Option<Vec<ValueRate>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValueRate {
    pub title: String,
    pub purchase: RateValue,
    pub sell: RateValue,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateValue {
    pub value: Decimal,
}

pub async fn get(client: &reqwest::Client, config: &Config) -> anyhow::Result<Response> {
    let html = client
        .get(config.rates_url.clone())
        .send()
        .await?
        .text()
        .await?;
    let document = Document::from(html.as_str());
    let div = document
        .find(Attr("data-component", "ExchangePage"))
        .next()
        .ok_or(Error::Html)?;
    let raw = div.attr("data-initial").ok_or(Error::Html)?;
    let resp = serde_json::from_str(raw)?;
    Ok(resp)
}
