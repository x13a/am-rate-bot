pub use crate::source::BaseConfig as Config;
use crate::source::{de, get_json_for_rate_type, Currency, Rate, RateType};
use rust_decimal::{serde::arbitrary_precision, Decimal};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase", transparent)]
pub struct Response {
    pub array_of_exchange_rate: Vec<ExchangeRate>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ExchangeRate {
    #[serde(deserialize_with = "de::currency")]
    pub currency: Currency,
    #[serde(deserialize_with = "arbitrary_precision::deserialize")]
    pub purchase: Decimal,
    #[serde(deserialize_with = "arbitrary_precision::deserialize")]
    pub sale: Decimal,
    #[serde(deserialize_with = "arbitrary_precision::deserialize")]
    pub rate_for: Decimal,
}

fn _collect(resp: Response, rate_type: RateType) -> Vec<Rate> {
    resp.array_of_exchange_rate
        .iter()
        .map(|v| Rate {
            from: v.currency.clone(),
            to: Currency::default(),
            rate_type,
            buy: Some(v.purchase / v.rate_for),
            sell: Some(v.sale / v.rate_for),
        })
        .collect()
}

pub async fn collect(client: &reqwest::Client, config: &Config) -> anyhow::Result<Vec<Rate>> {
    let mut results = vec![];
    for rate_type in [RateType::NoCash, RateType::Cash] {
        let resp: Response = get_json_for_rate_type(client, config, rate_type).await?;
        let rates = _collect(resp, rate_type);
        results.extend_from_slice(&rates);
    }
    Ok(results)
}
