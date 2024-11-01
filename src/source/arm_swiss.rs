pub use crate::source::BaseConfig as Config;
use crate::source::{get_json, Currency, Rate, RateType};
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Response {
    pub lmasbrate: Vec<LmasbRate>,
}

#[derive(Debug, Deserialize)]
pub struct LmasbRate {
    #[serde(rename = "ISO")]
    pub iso: String,
    #[serde(rename = "BID")]
    pub bid: Decimal,
    #[serde(rename = "OFFER")]
    pub offer: Decimal,
    #[serde(rename = "BID_cash")]
    pub bid_cash: Decimal,
    #[serde(rename = "OFFER_cash")]
    pub offer_cash: Decimal,
}

pub async fn collect(client: &reqwest::Client, config: &Config) -> anyhow::Result<Vec<Rate>> {
    let resp: Response = get_json(client, config).await?;
    let mut rates = vec![];
    let to = Currency::default();
    for rate in resp.lmasbrate {
        let mut ws = rate.iso.split_whitespace();
        let Some(iso) = ws.next() else {
            continue;
        };
        let from = Currency::new(iso);
        let mut amount = Decimal::ONE;
        if let Some(s) = ws.next() {
            let Ok(new_amount) = s.parse() else {
                continue;
            };
            amount = new_amount;
        }
        rates.push(Rate {
            from: from.clone(),
            to: to.clone(),
            rate_type: RateType::NoCash,
            buy: Some(rate.bid / amount),
            sell: Some(rate.offer / amount),
        });
        rates.push(Rate {
            from: from.clone(),
            to: to.clone(),
            rate_type: RateType::Cash,
            buy: Some(rate.bid_cash / amount),
            sell: Some(rate.offer_cash / amount),
        });
    }
    Ok(rates)
}
