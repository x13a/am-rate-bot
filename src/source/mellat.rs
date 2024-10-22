pub use crate::source::BaseConfig as Config;
use crate::source::{de, get_json, Currency, Rate, RateType};
use rust_decimal::{serde::arbitrary_precision, Decimal};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Response {
    pub result: Result,
}

#[derive(Debug, Deserialize)]
pub struct Data {
    #[serde(deserialize_with = "arbitrary_precision::deserialize")]
    pub buy: Decimal,
    #[serde(deserialize_with = "arbitrary_precision::deserialize")]
    pub sell: Decimal,
    #[serde(deserialize_with = "de::currency")]
    pub currency: Currency,
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

pub async fn collect(client: &reqwest::Client, config: &Config) -> anyhow::Result<Vec<Rate>> {
    let resp: Response = get_json(client, config).await?;
    let mut rates = vec![];
    let to = Currency::default();
    for rate in resp.result.data {
        let from = rate.currency;
        rates.push(Rate {
            from: from.clone(),
            to: to.clone(),
            rate_type: RateType::NoCash,
            buy: Some(rate.buy),
            sell: Some(rate.sell),
        });
        rates.push(Rate {
            from: from.clone(),
            to: to.clone(),
            rate_type: RateType::Cash,
            buy: Some(rate.buy_cash),
            sell: Some(rate.sell_cash),
        });
    }
    Ok(rates)
}
