pub use crate::source::BaseConfig as Config;
use crate::source::{de, get_json, Currency as ModCurrency, Rate, RateType};
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Response {
    pub data: Data,
}

#[derive(Debug, Deserialize)]
pub struct Data {
    pub currencies: Currencies,
}

#[derive(Debug, Deserialize)]
pub struct Currencies {
    pub cash: Vec<Currency>,
    pub no_cash: Vec<Currency>,
}

#[derive(Debug, Deserialize)]
pub struct Currency {
    #[serde(rename = "type", deserialize_with = "de::currency")]
    pub curr_type: ModCurrency,
    pub buy: Decimal,
    pub sell: Decimal,
}

pub async fn collect(client: &reqwest::Client, config: &Config) -> anyhow::Result<Vec<Rate>> {
    let resp: Response = get_json(client, config).await?;
    let mut results = vec![];
    for (rate_type, rates) in [
        (RateType::NoCash, resp.data.currencies.no_cash),
        (RateType::Cash, resp.data.currencies.cash),
    ] {
        let rates = rates
            .iter()
            .map(|v| Rate {
                from: v.curr_type.clone(),
                to: ModCurrency::default(),
                rate_type,
                buy: Some(v.buy),
                sell: Some(v.sell),
            })
            .collect::<Vec<_>>();
        results.extend_from_slice(&rates);
    }
    Ok(results)
}
