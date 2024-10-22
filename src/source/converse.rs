pub use crate::source::BaseConfig as Config;
use crate::source::{de, get_json, Currency as ModCurrency, Rate, RateType};
use rust_decimal::{serde::arbitrary_precision, Decimal};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Response {
    #[serde(rename = "Non Cash")]
    pub non_cash: Vec<Item>,
    #[serde(rename = "Cash")]
    pub cash: Vec<Item>,
}

#[derive(Debug, Deserialize)]
pub struct Item {
    #[serde(deserialize_with = "arbitrary_precision::deserialize")]
    pub buy: Decimal,
    #[serde(deserialize_with = "arbitrary_precision::deserialize")]
    pub sell: Decimal,
    #[serde(deserialize_with = "de::currency")]
    pub iso2: ModCurrency,
    pub currency: Currency,
}

#[derive(Debug, Deserialize)]
pub struct Currency {
    #[serde(deserialize_with = "de::currency")]
    pub iso: ModCurrency,
    pub use_for_rates: i32,
}

pub async fn collect(client: &reqwest::Client, config: &Config) -> anyhow::Result<Vec<Rate>> {
    let resp: Response = get_json(client, config).await?;
    let mut results = vec![];
    for (rate_type, rates) in [
        (RateType::NoCash, resp.non_cash),
        (RateType::Cash, resp.cash),
    ] {
        let rates = rates
            .iter()
            .filter(|v| v.currency.use_for_rates != 0)
            .map(|v| Rate {
                from: v.currency.iso.clone(),
                to: v.iso2.clone(),
                rate_type,
                buy: Some(v.buy),
                sell: Some(v.sell),
            })
            .collect::<Vec<_>>();
        results.extend_from_slice(&rates);
    }
    Ok(results)
}
