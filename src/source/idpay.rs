pub use crate::source::idbank::Response;
use crate::source::{idbank, percent, BaseConfigTrait, Currency, Error, Rate, RateType};
use anyhow::bail;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub rates_url: String,
    pub enabled: bool,
    pub commission_rate: Decimal,
    pub commission_rate_to_ru_card: Decimal,
}

impl BaseConfigTrait for Config {
    fn rates_url(&self) -> String {
        self.rates_url.clone()
    }
}

pub async fn collect(client: &reqwest::Client, config: &Config) -> anyhow::Result<Vec<Rate>> {
    let resp: Response = idbank::post(client, config).await?;
    let to = Currency::default();
    let from = Currency::rub();
    let Some(rate) = resp
        .result
        .currency_rate
        .iter()
        .filter(|v| v.iso_txt == from)
        .next()
    else {
        bail!(Error::NoRates);
    };
    let mut rate_buy = None;
    let mut rate_sell = None;
    let mut rate_buy_idbank = None;
    if let Some(buy) = rate.buy {
        if buy > Decimal::ZERO {
            rate_buy = Some(buy - percent(config.commission_rate, buy));
        }
    };
    if let Some(sell) = rate.sell {
        if sell > Decimal::ZERO {
            rate_sell = Some(
                sell + percent(
                    config.commission_rate + config.commission_rate_to_ru_card,
                    sell,
                ),
            );
        }
    }
    if let Some(sell) = rate.csh_sell_trf {
        if sell > Decimal::ZERO {
            rate_buy_idbank = Some(sell - percent(config.commission_rate, sell));
        }
    }
    Ok(vec![
        Rate {
            from: from.clone(),
            to: to.clone(),
            rate_type: RateType::NoCash,
            buy: rate_buy,
            sell: rate_sell,
        },
        Rate {
            from: from.clone(),
            to: to.clone(),
            rate_type: RateType::NoCash,
            buy: rate_buy_idbank,
            sell: None,
        },
    ])
}
