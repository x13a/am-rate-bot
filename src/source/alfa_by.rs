use crate::source::{percent, Currency, Error, Rate, RateType};
use anyhow::bail;
use regex::Regex;
use rust_decimal::Decimal;
use select::{document::Document, predicate::Attr};
use serde::Deserialize;
use std::{str::FromStr, sync::LazyLock};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub rates_url: String,
    pub enabled: bool,
    pub commission_rate: Decimal,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub initial_items: Vec<Item>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub currencies_data: Vec<CurrencyData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyData {
    pub value: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Value {
    pub exchange_rate: Vec<ValueRate>,
    #[serde(default)]
    pub conversion_rate: Option<Vec<ValueRate>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValueRate {
    pub title: String,
    pub purchase: RateValue,
    pub sell: RateValue,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateValue {
    pub value: Decimal,
}

async fn get(client: &reqwest::Client, config: &Config) -> anyhow::Result<Response> {
    let html = client
        .get(config.rates_url.clone())
        .send()
        .await?
        .text()
        .await?;
    let document = Document::from(html.as_str());
    let div = document
        .find(Attr("data-component", "ExchangePage"))
        .next()
        .ok_or(Error::Html)?;
    let raw = div.attr("data-initial").ok_or(Error::Html)?;
    let resp = serde_json::from_str(raw)?;
    Ok(resp)
}

pub async fn collect(client: &reqwest::Client, config: &Config) -> anyhow::Result<Vec<Rate>> {
    let resp: Response = get(client, config).await?;
    let Some(item) = resp.initial_items.iter().next() else {
        bail!(Error::NoRates);
    };
    let Some(data) = item.currencies_data.iter().next() else {
        bail!(Error::NoRates);
    };
    let mut rates = vec![];
    for rate in &data.value.exchange_rate {
        let Ok((amount, from)) = regex_helper(&rate.title) else {
            continue;
        };
        let (buy, sell) = match from.0.as_str() {
            Currency::RUB => (Some(rate.purchase.value / amount), None),
            Currency::USD | Currency::EUR => {
                let sell = rate.sell.value / amount;
                (None, Some(sell + percent(config.commission_rate, sell)))
            }
            _ => continue,
        };
        rates.push(Rate {
            from,
            to: Currency::new("BYN"),
            rate_type: RateType::NoCash,
            buy,
            sell,
        });
    }
    if let Some(conversation_rate) = &data.value.conversion_rate {
        for rate in conversation_rate {
            let Some((from, to)) = rate.title.split_once('/') else {
                continue;
            };
            let (buy, sell) = match (from, to) {
                (Currency::USD, Currency::RUB) | (Currency::EUR, Currency::RUB) => {
                    let sell = rate.sell.value;
                    (None, Some(sell + percent(config.commission_rate, sell)))
                }
                _ => continue,
            };
            rates.push(Rate {
                from: Currency::new(from),
                to: Currency::new(to),
                rate_type: RateType::NoCash,
                buy,
                sell,
            });
        }
    };
    Ok(rates)
}

fn regex_helper(s: &str) -> anyhow::Result<(Decimal, Currency)> {
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?P<amount>\d+).*\((?P<iso>[A-Z]{3})\)").unwrap());
    let caps = RE.captures(s).ok_or(Error::NoRates)?;
    let amount = Decimal::from_str(&caps["amount"])?;
    let from = Currency::new(&caps["iso"]);
    Ok((amount, from))
}
