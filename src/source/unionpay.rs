pub use crate::source::BaseConfig as Config;
use crate::source::{de, get_json, Currency, Error, Rate, RateType};
use anyhow::bail;
use chrono::{Datelike, Days, Utc, Weekday};
use rust_decimal::{serde::arbitrary_precision, Decimal};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub exchange_rate_json: Vec<ExchangeRate>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeRate {
    #[serde(deserialize_with = "de::currency")]
    trans_cur: Currency,
    #[serde(deserialize_with = "de::currency")]
    base_cur: Currency,
    #[serde(deserialize_with = "arbitrary_precision::deserialize")]
    rate_data: Decimal,
}

pub async fn collect(client: &reqwest::Client, config: &Config) -> anyhow::Result<Vec<Rate>> {
    let mut date = Utc::now().date_naive();
    let days = match date.weekday() {
        Weekday::Sat => 1,
        Weekday::Sun => 2,
        _ => 0,
    };
    date = date.checked_sub_days(Days::new(days)).unwrap();
    let mut config = config.clone();
    config.rates_url = config
        .rates_url
        .replace("%s", date.format("%Y%m%d").to_string().as_str());
    let resp: Response = get_json(client, &config).await?;
    let from = Currency::usd();
    let to = Currency::rub();
    let Some(rate) = resp
        .exchange_rate_json
        .iter()
        .filter(|v| v.base_cur == from && v.trans_cur == to)
        .next()
    else {
        bail!(Error::NoRates);
    };
    let buy = if rate.rate_data > Decimal::ZERO {
        Some(Decimal::ONE / rate.rate_data)
    } else {
        None
    };
    let new_rate = |rate_type: RateType| Rate {
        from: from.clone(),
        to: to.clone(),
        rate_type,
        buy,
        sell: None,
    };
    Ok(vec![new_rate(RateType::NoCash), new_rate(RateType::Cash)])
}
