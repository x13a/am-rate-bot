pub use crate::source::unibank::Response;
use crate::source::{percent, unibank, BaseConfigTrait, Currency, Error, Rate, RateType};
use anyhow::bail;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub rates_url: String,
    pub enabled: bool,
    pub commission_rate: Decimal,
}

impl BaseConfigTrait for Config {
    fn rates_url(&self) -> String {
        self.rates_url.clone()
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
    Ok(vec![Rate {
        from: from.clone(),
        to: Currency::default(),
        rate_type: RateType::NoCash,
        buy: Some(buy - percent(config.commission_rate, buy)),
        sell: None,
    }])
}
