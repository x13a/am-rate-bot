pub use crate::source::BaseConfig as Config;
use crate::source::{de, get_json, Currency, Rate as ModRate, RateType};
use rust_decimal::{serde::arbitrary_precision_option, Decimal};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Response {
    pub items: Vec<Item>,
}

#[derive(Debug, Deserialize)]
pub struct Item {
    #[serde(deserialize_with = "de::currency")]
    pub code: Currency,
    pub cash: Rate,
    pub cashless: Rate,
}

#[derive(Debug, Deserialize)]
pub struct Rate {
    #[serde(deserialize_with = "arbitrary_precision_option::deserialize")]
    pub buy: Option<Decimal>,
    #[serde(deserialize_with = "arbitrary_precision_option::deserialize")]
    pub sell: Option<Decimal>,
}

pub async fn collect(client: &reqwest::Client, config: &Config) -> anyhow::Result<Vec<ModRate>> {
    let resp: Response = get_json(client, config).await?;
    let mut rates = vec![];
    let to = Currency::default();
    for item in resp.items {
        let from = item.code;
        rates.push(ModRate {
            from: from.clone(),
            to: to.clone(),
            rate_type: RateType::NoCash,
            buy: item.cashless.buy,
            sell: item.cashless.sell,
        });
        rates.push(ModRate {
            from: from.clone(),
            to: to.clone(),
            rate_type: RateType::Cash,
            buy: item.cash.buy,
            sell: item.cash.sell,
        });
    }
    Ok(rates)
}
