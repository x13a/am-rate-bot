pub use crate::source::BaseConfig as Config;
use crate::source::{de, get_json, Currency, Rate as ModRate, RateType};
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Response {
    pub result: Result,
}

#[derive(Debug, Deserialize)]
pub struct Result {
    pub rates: Rates,
}

#[derive(Debug, Deserialize)]
pub struct Rates {
    pub cash: Vec<Rate>,
    pub non_cash: Vec<Rate>,
    pub cross: Vec<CrossRate>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Rate {
    pub buy: Decimal,
    pub sell: Decimal,
    #[serde(deserialize_with = "de::currency")]
    pub currency: Currency,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CrossRate {
    pub buy: Decimal,
    pub sell: Decimal,
    pub currency: String,
}

pub async fn collect(client: &reqwest::Client, config: &Config) -> anyhow::Result<Vec<ModRate>> {
    let resp: Response = get_json(client, config).await?;
    let rates = parse(resp)?;
    Ok(rates)
}

pub(crate) fn parse(resp: Response) -> anyhow::Result<Vec<ModRate>> {
    let mut results = vec![];
    for (rate_type, rates) in [
        (RateType::NoCash, resp.result.rates.non_cash),
        (RateType::Cash, resp.result.rates.cash),
    ] {
        let rates = rates
            .iter()
            .map(|v| ModRate {
                from: v.currency.clone(),
                to: Currency::default(),
                rate_type,
                buy: Some(v.buy),
                sell: Some(v.sell),
            })
            .collect::<Vec<_>>();
        results.extend_from_slice(&rates);
    }
    for rate in resp.result.rates.cross {
        let Some((from, to)) = rate.currency.split_once('/') else {
            continue;
        };
        results.push(ModRate {
            from: Currency::new(from),
            to: Currency::new(to),
            rate_type: RateType::NoCash,
            buy: Some(rate.buy),
            sell: Some(rate.sell),
        });
    }
    Ok(results)
}
