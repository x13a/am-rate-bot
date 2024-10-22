pub use crate::source::BaseConfig as Config;
use crate::source::{de, get_json_for_rate_type, Currency, Rate as ModRate, RateType};
use rust_decimal::{serde::arbitrary_precision, Decimal};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Response {
    pub rates: Vec<Rate>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Rate {
    #[serde(deserialize_with = "arbitrary_precision::deserialize")]
    pub buy: Decimal,
    #[serde(deserialize_with = "de::currency")]
    pub id: Currency,
    #[serde(deserialize_with = "arbitrary_precision::deserialize")]
    pub sale: Decimal,
    #[serde(deserialize_with = "arbitrary_precision::deserialize")]
    pub unit: Decimal,
}

pub async fn collect(client: &reqwest::Client, config: &Config) -> anyhow::Result<Vec<ModRate>> {
    let mut results = vec![];
    for rate_type in [RateType::NoCash, RateType::Cash] {
        let resp: Response = get_json_for_rate_type(client, config, rate_type).await?;
        let rates = resp
            .rates
            .iter()
            .map(|v| ModRate {
                from: v.id.clone(),
                to: Currency::default(),
                rate_type,
                buy: Some(v.buy / v.unit),
                sell: Some(v.sale / v.unit),
            })
            .collect::<Vec<_>>();
        results.extend_from_slice(&rates);
    }
    Ok(results)
}
