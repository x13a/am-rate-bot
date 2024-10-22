pub use crate::source::BaseConfig as Config;
use crate::source::{de, get_json, Currency, Rate as ModRate, RateType};
use rust_decimal::{serde::arbitrary_precision_option, Decimal};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    #[serde(deserialize_with = "de::currency")]
    pub main_currency_code: Currency,
    pub rate_currency_settings: Vec<Item>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    #[serde(deserialize_with = "de::currency")]
    pub currency_code: Currency,
    pub rates: Vec<Rate>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rate {
    #[serde(deserialize_with = "arbitrary_precision_option::deserialize")]
    pub buy_rate: Option<Decimal>,
    #[serde(deserialize_with = "arbitrary_precision_option::deserialize")]
    pub sell_rate: Option<Decimal>,
    #[serde(rename = "type", deserialize_with = "de::rate_type")]
    pub rate_type: RateType,
}

pub async fn collect(client: &reqwest::Client, config: &Config) -> anyhow::Result<Vec<ModRate>> {
    let resp: Response = get_json(client, config).await?;
    let mut rates = vec![];
    for item in resp.rate_currency_settings {
        for rate in item
            .rates
            .iter()
            .filter(|v| [RateType::NoCash, RateType::Cash].contains(&v.rate_type))
        {
            rates.push(ModRate {
                from: item.currency_code.clone(),
                to: resp.main_currency_code.clone(),
                rate_type: rate.rate_type,
                buy: rate.buy_rate,
                sell: rate.sell_rate,
            });
        }
    }
    Ok(rates)
}
