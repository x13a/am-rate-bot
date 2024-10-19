use crate::sources::{
    acba, aeb, alfa_by, ameria, amio, ararat, ardshin, arm_swiss, armsoft, artsakh, avosend,
    byblos, cb_am, converse, evoca, fast, hsbc, idbank, idpay, ineco, kwikpay, lsoft, mellat, mir,
    moex, sas, unibank, unistream, vtb_am, Config, Currency, JsonResponse, LSoftResponse, Rate,
    RateType, RateTypeJsonResponse, Source,
};
use anyhow::bail;
use regex::Regex;
use reqwest::Client;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::{collections::HashMap, env, fmt::Debug, str::FromStr, sync::LazyLock};
use strum::{EnumCount, IntoEnumIterator};
use tokio::sync::mpsc;

pub async fn collect_all(
    client: &Client,
    config: &Config,
) -> HashMap<Source, anyhow::Result<Vec<Rate>>> {
    let mut results = HashMap::new();
    let (tx, mut rx) = mpsc::channel(Source::COUNT);
    for src in Source::iter() {
        if !config.is_enabled_for(src) {
            continue;
        }
        match src {
            Source::MoEx => {
                if env::var(moex::ENV_TINKOFF_TOKEN)
                    .unwrap_or_default()
                    .is_empty()
                {
                    continue;
                }
            }
            _ => {}
        }
        let client = client.clone();
        let config = config.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            let result = collect(&client, &config, src).await;
            tx.send((src, result)).await.unwrap();
        });
    }
    drop(tx);
    while let Some(result) = rx.recv().await {
        results.insert(result.0, result.1);
    }
    results
}

pub fn filter_collection(
    results: HashMap<Source, anyhow::Result<Vec<Rate>>>,
) -> HashMap<Source, Vec<Rate>> {
    let mut rates = HashMap::new();
    for (src, result) in results {
        match result {
            Ok(v) => {
                if v.is_empty() {
                    continue;
                }
                let v = v
                    .iter()
                    .filter(|v| {
                        (!v.from.is_empty() && !v.to.is_empty())
                            && (v.buy.is_some_and(|v| v > dec!(0.0))
                                || v.sell.is_some_and(|v| v > dec!(0.0)))
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                rates.insert(src, v);
            }
            Err(err) => log::error!("failed to get rate: {err}, src: {src}"),
        }
    }
    rates
}

async fn collect(client: &Client, config: &Config, src: Source) -> anyhow::Result<Vec<Rate>> {
    let rates = match src {
        Source::Acba => collect_acba(client, &config.acba).await?,
        Source::Ameria => collect_ameria(client, &config.ameria).await?,
        Source::Ardshin => collect_ardshin(client, &config.ardshin).await?,
        Source::ArmSwiss => collect_arm_swiss(client, &config.arm_swiss).await?,
        Source::CbAm => collect_cb_am(client, &config.cb_am).await?,
        Source::Evoca => collect_evoca(client, &config.evoca).await?,
        Source::Fast => collect_fast(client, &config.fast).await?,
        Source::Ineco => collect_ineco(client, &config.ineco).await?,
        Source::Mellat => collect_mellat(client, &config.mellat).await?,
        Source::Converse => collect_converse(client, &config.converse).await?,
        Source::AEB => collect_aeb(client, &config.aeb).await?,
        Source::VtbAm => collect_vtb_am(client, &config.vtb_am).await?,
        Source::Artsakh => collect_artsakh(client, &config.artsakh).await?,
        Source::Unibank => collect_unibank(client, &config.unibank).await?,
        Source::Amio => collect_amio(client, &config.amio).await?,
        Source::Byblos => collect_byblos(client, &config.byblos).await?,
        Source::IdBank => collect_idbank(client, &config.idbank).await?,
        Source::MoEx => collect_moex(client, &config.moex).await?,
        Source::Ararat => collect_ararat(client, &config.ararat).await?,
        Source::IdPay => collect_idpay(client, &config.idpay).await?,
        Source::Mir => collect_mir(client, &config.mir).await?,
        Source::SAS => collect_sas(client, &config.sas).await?,
        Source::HSBC => collect_hsbc(client, &config.hsbc).await?,
        Source::Avosend => collect_avosend(client, &config.avosend).await?,
        Source::Kwikpay => collect_kwikpay(client, &config.kwikpay).await?,
        Source::Unistream => collect_unistream(client, &config.unistream).await?,
        Source::AlfaBy => collect_alfa_by(client, &config.alfa_by).await?,
    };
    Ok(rates)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("no rates found")]
    NoRates,
}

async fn collect_acba(client: &Client, config: &acba::Config) -> anyhow::Result<Vec<Rate>> {
    let resp: acba::Response = acba::Response::get(client, config).await?;
    let rates = parse_acba(resp)?;
    Ok(rates)
}

pub(crate) fn parse_acba(resp: acba::Response) -> anyhow::Result<Vec<Rate>> {
    let mut results = vec![];
    for (rate_type, rates) in [
        (RateType::NoCash, resp.result.rates.non_cash),
        (RateType::Cash, resp.result.rates.cash),
    ] {
        let rates = rates
            .iter()
            .map(|v| Rate {
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
        let currency = rate.currency.to_string();
        let Some((from, to)) = currency.split_once('/') else {
            continue;
        };
        results.push(Rate {
            from: Currency::new(from),
            to: Currency::new(to),
            rate_type: RateType::NoCash,
            buy: Some(rate.buy),
            sell: Some(rate.sell),
        });
    }
    Ok(results)
}

async fn collect_ameria(client: &Client, config: &ameria::Config) -> anyhow::Result<Vec<Rate>> {
    let mut results = vec![];
    for rate_type in [RateType::NoCash, RateType::Cash] {
        let resp: armsoft::Response = ameria::Response::get(client, config, rate_type).await?;
        let rates = collect_armsoft(resp, rate_type);
        results.extend_from_slice(&rates);
    }
    Ok(results)
}

fn collect_armsoft(resp: armsoft::Response, rate_type: RateType) -> Vec<Rate> {
    resp.array_of_exchange_rate
        .iter()
        .map(|v| Rate {
            from: v.currency.clone(),
            to: Currency::default(),
            rate_type,
            buy: Some(v.purchase / v.rate_for),
            sell: Some(v.sale / v.rate_for),
        })
        .collect()
}

async fn collect_ardshin(client: &Client, config: &ardshin::Config) -> anyhow::Result<Vec<Rate>> {
    let resp: ardshin::Response = ardshin::Response::get(client, config).await?;
    let mut results = vec![];
    for (rate_type, rates) in [
        (RateType::NoCash, resp.data.currencies.no_cash),
        (RateType::Cash, resp.data.currencies.cash),
    ] {
        let rates = rates
            .iter()
            .map(|v| Rate {
                from: v.curr_type.clone(),
                to: Currency::default(),
                rate_type,
                buy: Some(v.buy),
                sell: Some(v.sell),
            })
            .collect::<Vec<_>>();
        results.extend_from_slice(&rates);
    }
    Ok(results)
}

async fn collect_arm_swiss(
    client: &Client,
    config: &arm_swiss::Config,
) -> anyhow::Result<Vec<Rate>> {
    let resp: arm_swiss::Response = arm_swiss::Response::get(client, config).await?;
    let mut rates = vec![];
    let to = Currency::default();
    for rate in resp.lmasbrate {
        let mut ws = rate.iso.split_whitespace();
        let Some(iso) = ws.next() else {
            continue;
        };
        let from = Currency::new(iso);
        let mut amount = dec!(1.0);
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

async fn collect_cb_am(client: &Client, config: &cb_am::Config) -> anyhow::Result<Vec<Rate>> {
    let resp = cb_am::Response::get(client, config).await?;
    let rates = resp
        .soap_body
        .exchange_rates_latest_response
        .exchange_rates_latest_result
        .rates
        .exchange_rate
        .iter()
        .map(|v| {
            let rate = Some(v.rate / v.amount);
            Rate {
                from: v.iso.clone(),
                to: Currency::default(),
                rate_type: RateType::Cb,
                buy: rate,
                sell: rate,
            }
        })
        .collect();
    Ok(rates)
}

async fn collect_evoca(client: &Client, config: &evoca::Config) -> anyhow::Result<Vec<Rate>> {
    let mut results = vec![];
    for rate_type in [RateType::NoCash, RateType::Cash] {
        let resp: armsoft::Response = evoca::Response::get(client, config, rate_type).await?;
        let rates = collect_armsoft(resp, rate_type);
        results.extend_from_slice(&rates);
    }
    Ok(results)
}

async fn collect_fast(client: &Client, config: &fast::Config) -> anyhow::Result<Vec<Rate>> {
    let mut results = vec![];
    for rate_type in [RateType::NoCash, RateType::Cash] {
        let resp: fast::Response = fast::Response::get(client, config, rate_type).await?;
        let rates = resp
            .rates
            .iter()
            .map(|v| Rate {
                from: v.id.clone(),
                to: Currency::default(),
                rate_type,
                buy: Some(v.buy / v.unit),
                sell: Some(v.sale / v.unit),
            })
            .collect::<Vec<_>>();
        results.extend_from_slice(&rates);
    }
    Ok(results)
}

async fn collect_ineco(client: &Client, config: &ineco::Config) -> anyhow::Result<Vec<Rate>> {
    let resp: ineco::Response = ineco::Response::get(client, config).await?;
    let mut rates = vec![];
    let to = Currency::default();
    for item in resp.items {
        let from = item.code;
        rates.push(Rate {
            from: from.clone(),
            to: to.clone(),
            rate_type: RateType::NoCash,
            buy: item.cashless.buy,
            sell: item.cashless.sell,
        });
        rates.push(Rate {
            from: from.clone(),
            to: to.clone(),
            rate_type: RateType::Cash,
            buy: item.cash.buy,
            sell: item.cash.sell,
        });
    }
    Ok(rates)
}

async fn collect_mellat(client: &Client, config: &mellat::Config) -> anyhow::Result<Vec<Rate>> {
    let resp: mellat::Response = mellat::Response::get(client, config).await?;
    let mut rates = vec![];
    let to = Currency::default();
    for rate in resp.result.data {
        let from = rate.currency;
        rates.push(Rate {
            from: from.clone(),
            to: to.clone(),
            rate_type: RateType::NoCash,
            buy: Some(rate.buy),
            sell: Some(rate.sell),
        });
        rates.push(Rate {
            from: from.clone(),
            to: to.clone(),
            rate_type: RateType::Cash,
            buy: Some(rate.buy_cash),
            sell: Some(rate.sell_cash),
        });
    }
    Ok(rates)
}

async fn collect_converse(client: &Client, config: &converse::Config) -> anyhow::Result<Vec<Rate>> {
    let resp: converse::Response = converse::Response::get(client, config).await?;
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

async fn collect_aeb(client: &Client, config: &aeb::Config) -> anyhow::Result<Vec<Rate>> {
    let resp: aeb::Response = aeb::Response::get(client, config).await?;
    let mut rates = vec![];
    for item in resp.rate_currency_settings {
        for rate in item
            .rates
            .iter()
            .filter(|v| [RateType::NoCash, RateType::Cash].contains(&v.rate_type))
        {
            rates.push(Rate {
                from: item.currency_code.clone(),
                to: resp.main_currency_code.clone(),
                rate_type: rate.rate_type,
                buy: rate.buy_rate,
                sell: rate.sell_rate,
            });
        }
    }
    Ok(rates)
}

async fn collect_vtb_am(client: &Client, config: &vtb_am::Config) -> anyhow::Result<Vec<Rate>> {
    let resp: vtb_am::Response = vtb_am::Response::get(client, config).await?;
    Ok(resp.rates)
}

async fn collect_artsakh(client: &Client, config: &artsakh::Config) -> anyhow::Result<Vec<Rate>> {
    let resp: lsoft::Response = artsakh::Response::get(client, config).await?;
    let rates = collect_lsoft(resp)?;
    Ok(rates)
}

async fn collect_unibank(client: &Client, config: &unibank::Config) -> anyhow::Result<Vec<Rate>> {
    let resp: lsoft::Response = unibank::Response::get(client, config).await?;
    let rates = collect_lsoft(resp)?;
    Ok(rates)
}

fn collect_lsoft(resp: lsoft::Response) -> anyhow::Result<Vec<Rate>> {
    let mut rates = vec![];
    let to = Currency::default();
    for item in resp.get_currency_list.currency_list {
        let from = item.external_id;
        rates.push(Rate {
            from: from.clone(),
            to: to.clone(),
            rate_type: RateType::NoCash,
            buy: item.buy,
            sell: item.sell,
        });
        rates.push(Rate {
            from: from.clone(),
            to: to.clone(),
            rate_type: RateType::Cash,
            buy: item.csh_buy,
            sell: item.csh_sell,
        });
    }
    Ok(rates)
}

async fn collect_amio(client: &Client, config: &amio::Config) -> anyhow::Result<Vec<Rate>> {
    let mut results = vec![];
    for rate_type in [RateType::NoCash, RateType::Cash] {
        let resp: armsoft::Response = amio::Response::get(client, config, rate_type).await?;
        let rates = collect_armsoft(resp, rate_type);
        results.extend_from_slice(&rates);
    }
    Ok(results)
}

async fn collect_byblos(client: &Client, config: &byblos::Config) -> anyhow::Result<Vec<Rate>> {
    let mut results = vec![];
    for rate_type in [RateType::NoCash, RateType::Cash] {
        let resp: armsoft::Response = byblos::Response::get(client, config, rate_type).await?;
        let rates = collect_armsoft(resp, rate_type);
        results.extend_from_slice(&rates);
    }
    Ok(results)
}

async fn collect_idbank(client: &Client, config: &idbank::Config) -> anyhow::Result<Vec<Rate>> {
    let resp: idbank::Response = idbank::Response::get(client, config).await?;
    let mut rates = vec![];
    let to = Currency::default();
    for rate in resp.result.currency_rate {
        let from = rate.iso_txt;
        rates.push(Rate {
            from: from.clone(),
            to: to.clone(),
            rate_type: RateType::NoCash,
            buy: rate.buy,
            sell: rate.sell,
        });
        rates.push(Rate {
            from: from.clone(),
            to: to.clone(),
            rate_type: RateType::Cash,
            buy: rate.csh_buy,
            sell: rate.csh_sell,
        });
    }
    Ok(rates)
}

async fn collect_moex(client: &Client, config: &moex::Config) -> anyhow::Result<Vec<Rate>> {
    let currency: moex::CurrencyResponse = moex::CurrencyResponse::get(client, config).await?;
    let order_book: moex::GetOrderBookResponse =
        moex::GetOrderBookResponse::get(client, config).await?;
    let to_decimal = |units: String, nano: i32| format!("{}.{}", units, nano).parse::<Decimal>();
    let mut rate_buy = None;
    let mut rate_sell = None;
    let nominal = to_decimal(
        currency.instrument.nominal.units,
        currency.instrument.nominal.nano,
    )?;
    if let Some(bid) = order_book.bids.first() {
        let sell = to_decimal(bid.price.units.clone(), bid.price.nano)?;
        if sell > dec!(0.0) {
            rate_sell = Some(nominal / sell);
        }
    }
    if let Some(ask) = order_book.asks.first() {
        let buy = to_decimal(ask.price.units.clone(), ask.price.nano)?;
        if buy > dec!(0.0) {
            rate_buy = Some(nominal / buy);
        }
    }
    let mut rates = vec![];
    if rate_buy.is_some() || rate_sell.is_some() {
        rates.push(Rate {
            from: Currency::rub(),
            to: Currency::default(),
            rate_type: RateType::NoCash,
            buy: rate_buy,
            sell: rate_sell,
        });
    }
    Ok(rates)
}

async fn collect_ararat(client: &Client, config: &ararat::Config) -> anyhow::Result<Vec<Rate>> {
    let mut results = vec![];
    for rate_type in [RateType::NoCash, RateType::Cash] {
        let resp: armsoft::Response = ararat::Response::get(client, config, rate_type).await?;
        let rates = collect_armsoft(resp, rate_type);
        results.extend_from_slice(&rates);
    }
    Ok(results)
}

async fn collect_idpay(client: &Client, config: &idpay::Config) -> anyhow::Result<Vec<Rate>> {
    let resp: idpay::Response = idpay::Response::get(client, config).await?;
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
        if buy > dec!(0.0) {
            rate_buy = Some(buy - percent(config.commission_rate, buy));
        }
    };
    if let Some(sell) = rate.sell {
        if sell > dec!(0.0) {
            rate_sell = Some(
                sell + percent(
                    config.commission_rate + config.commission_rate_to_ru_card,
                    sell,
                ),
            );
        }
    }
    if let Some(sell) = rate.csh_sell_trf {
        if sell > dec!(0.0) {
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

fn percent(value: Decimal, from: Decimal) -> Decimal {
    value / dec!(100.0) * from
}

async fn collect_mir(client: &Client, config: &mir::Config) -> anyhow::Result<Vec<Rate>> {
    let resp: mir::Response = mir::Response::get(client, config).await?;
    let to = Currency::default();
    let Some(rate) = resp
        .content
        .iter()
        .filter(|v| v.currency.strcode == to)
        .next()
    else {
        bail!(Error::NoRates);
    };
    let buy = if rate.value_sell > dec!(0.0) {
        Some(dec!(1.0) / rate.value_sell)
    } else {
        None
    };
    let sell = if rate.value_buy > dec!(0.0) {
        Some(dec!(1.0) / rate.value_buy)
    } else {
        None
    };
    let new_rate = |rate_type: RateType| Rate {
        from: Currency::rub(),
        to: to.clone(),
        rate_type,
        buy,
        sell,
    };
    Ok(vec![new_rate(RateType::NoCash), new_rate(RateType::Cash)])
}

async fn collect_sas(client: &Client, config: &sas::Config) -> anyhow::Result<Vec<Rate>> {
    let resp: sas::Response = sas::Response::get(client, config).await?;
    Ok(resp.rates)
}

async fn collect_hsbc(client: &Client, config: &hsbc::Config) -> anyhow::Result<Vec<Rate>> {
    let resp: hsbc::Response = hsbc::Response::get(client, config).await?;
    Ok(resp.rates)
}

async fn collect_avosend(client: &Client, config: &avosend::Config) -> anyhow::Result<Vec<Rate>> {
    let resp: avosend::Response = avosend::Response::get(client, config).await?;
    Ok(vec![Rate {
        from: Currency::rub(),
        to: Currency::default(),
        rate_type: RateType::NoCash,
        buy: Some(resp.convert_rate),
        sell: None,
    }])
}

async fn collect_kwikpay(client: &Client, config: &kwikpay::Config) -> anyhow::Result<Vec<Rate>> {
    let resp: lsoft::Response = kwikpay::Response::get(client, config).await?;
    let rates = collect_lsoft(resp)?;
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

async fn collect_unistream(
    client: &Client,
    config: &unistream::Config,
) -> anyhow::Result<Vec<Rate>> {
    let resp: lsoft::Response = unistream::Response::get(client, config).await?;
    let rates = collect_lsoft(resp)?;
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

async fn collect_alfa_by(client: &Client, config: &alfa_by::Config) -> anyhow::Result<Vec<Rate>> {
    let resp: alfa_by::Response = alfa_by::Response::get(client, config).await?;
    let Some(item) = resp.initial_items.iter().next() else {
        bail!(Error::NoRates);
    };
    let Some(data) = item.currencies_data.iter().next() else {
        bail!(Error::NoRates);
    };
    let mut rates = vec![];
    for rate in &data.value.exchange_rate {
        let Ok((amount, from)) = alfa_by_regex_helper(&rate.title) else {
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

fn alfa_by_regex_helper(s: &str) -> anyhow::Result<(Decimal, Currency)> {
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?P<amount>\d+).*\((?P<iso>[A-Z]{3})\)").unwrap());
    let caps = RE.captures(s).ok_or(Error::NoRates)?;
    let amount = Decimal::from_str(&caps["amount"])?;
    let from = Currency::new(&caps["iso"]);
    Ok((amount, from))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    const TIMEOUT: u64 = 10;

    fn build_client() -> reqwest::Result<Client> {
        reqwest::ClientBuilder::new()
            .timeout(Duration::from_secs(TIMEOUT))
            .build()
    }

    fn load_src_config() -> anyhow::Result<Config> {
        let cfg = toml::from_str(include_str!("../config/src.toml"))?;
        Ok(cfg)
    }

    #[tokio::test]
    async fn test_collect_acba() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_src_config()?;
        collect(&client, &config, Source::Acba).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_ameria() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_src_config()?;
        collect(&client, &config, Source::Ameria).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_ardshin() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_src_config()?;
        collect(&client, &config, Source::Ardshin).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_arm_swiss() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_src_config()?;
        collect(&client, &config, Source::ArmSwiss).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_cb_am() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_src_config()?;
        collect(&client, &config, Source::CbAm).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_evoca() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_src_config()?;
        collect(&client, &config, Source::Evoca).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_fast() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_src_config()?;
        collect(&client, &config, Source::Fast).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_ineco() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_src_config()?;
        collect(&client, &config, Source::Ineco).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_mellat() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_src_config()?;
        collect(&client, &config, Source::Mellat).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_converse() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_src_config()?;
        collect(&client, &config, Source::Converse).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_aeb() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_src_config()?;
        collect(&client, &config, Source::AEB).await?;
        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_collect_vtb_am() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_src_config()?;
        collect(&client, &config, Source::VtbAm).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_artsakh() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_src_config()?;
        collect(&client, &config, Source::Artsakh).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_unibank() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_src_config()?;
        collect(&client, &config, Source::Unibank).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_amio() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_src_config()?;
        collect(&client, &config, Source::Amio).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_byblos() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_src_config()?;
        collect(&client, &config, Source::Byblos).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_idbank() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_src_config()?;
        collect(&client, &config, Source::IdBank).await?;
        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_collect_moex() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_src_config()?;
        collect(&client, &config, Source::MoEx).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_ararat() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_src_config()?;
        collect(&client, &config, Source::Ararat).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_idpay() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_src_config()?;
        collect(&client, &config, Source::IdPay).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_mir() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_src_config()?;
        collect(&client, &config, Source::Mir).await?;
        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_collect_sas() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_src_config()?;
        collect(&client, &config, Source::SAS).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_hsbc() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_src_config()?;
        collect(&client, &config, Source::HSBC).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_avosend() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_src_config()?;
        collect(&client, &config, Source::Avosend).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_kwikpay() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_src_config()?;
        collect(&client, &config, Source::Kwikpay).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_unistream() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_src_config()?;
        collect(&client, &config, Source::Unistream).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_alfa_by() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_src_config()?;
        collect(&client, &config, Source::AlfaBy).await?;
        Ok(())
    }
}
