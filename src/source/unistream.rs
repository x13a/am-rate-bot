pub use crate::source::unibank::Response;
use crate::source::{percent, unibank, BaseConfigTrait, Currency, Error, Rate, RateType};
use anyhow::bail;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub rates_url: String,
    pub enabled: bool,
    pub commission_rate_from_bank: Decimal,
    pub commission_rate_from_any_card: Decimal,
}

impl BaseConfigTrait for Config {
    fn rates_url(&self) -> &str {
        &self.rates_url
    }
}

pub async fn collect(client: &reqwest::Client, config: &Config) -> anyhow::Result<Vec<Rate>> {
    let rates = unibank::collect(client, config).await?;
    let from = Currency::rub();
    let Some(rate) = rates
        .iter()
        .filter(|v| v.rate_type == RateType::Cash && v.from == from)
        .next()
    else {
        bail!(Error::NoRates);
    };
    let Some(buy) = rate.buy else {
        bail!(Error::NoRates);
    };
    let results = [
        config.commission_rate_from_bank,
        config.commission_rate_from_any_card,
    ]
    .iter()
    .map(|v| Rate {
        from: from.clone(),
        to: Currency::default(),
        rate_type: RateType::NoCash,
        buy: Some(buy - percent(*v, buy)),
        sell: None,
    })
    .collect();
    Ok(results)
}
