pub use crate::source::BaseConfig as Config;
use crate::source::{de, get_json, Currency as ModCurrency, Error, Rate, RateType};
use anyhow::bail;
use rust_decimal::{serde::arbitrary_precision, Decimal};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Response {
    pub content: Vec<Content>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    pub currency: Currency,
    #[serde(deserialize_with = "arbitrary_precision::deserialize")]
    pub value_buy: Decimal,
    #[serde(deserialize_with = "arbitrary_precision::deserialize")]
    pub value_sell: Decimal,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Currency {
    #[serde(deserialize_with = "de::currency")]
    pub strcode: ModCurrency,
}

pub async fn collect(client: &reqwest::Client, config: &Config) -> anyhow::Result<Vec<Rate>> {
    let resp: Response = get_json(client, config).await?;
    let to = ModCurrency::default();
    let Some(rate) = resp
        .content
        .iter()
        .filter(|v| v.currency.strcode == to)
        .next()
    else {
        bail!(Error::NoRates);
    };
    let buy = if rate.value_sell > Decimal::ZERO {
        Some(Decimal::ONE / rate.value_sell)
    } else {
        None
    };
    let sell = if rate.value_buy > Decimal::ZERO {
        Some(Decimal::ONE / rate.value_buy)
    } else {
        None
    };
    let new_rate = |rate_type: RateType| Rate {
        from: ModCurrency::rub(),
        to: to.clone(),
        rate_type,
        buy,
        sell,
    };
    Ok(vec![new_rate(RateType::NoCash), new_rate(RateType::Cash)])
}
