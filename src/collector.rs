use crate::sources::{
    acba, aeb, ameria, amio, ararat, ardshin, arm_swiss, armsoft, artsakh, byblos, cb_am, converse,
    evoca, fast, hsbc, idbank, ineco, lsoft, mellat, mir, moex, sas, unibank, vtb_am, Currency,
    Rate, RateType, Source, SourceAphenaTrait, SourceCashUrlTrait, SourceSingleUrlTrait,
};
use reqwest::Client;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::fmt::Debug;
use strum::IntoEnumIterator;
use tokio::sync::mpsc;

pub async fn collect_all(client: &Client) -> HashMap<Source, anyhow::Result<Vec<Rate>>> {
    let mut results = HashMap::new();
    let (tx, mut rx) = mpsc::channel(Source::iter().count());
    for src in Source::iter() {
        let client = client.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            let result = collect(&client, src).await;
            tx.send((src, result)).await.expect("panic");
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
                    log::warn!("no rate found, source: {}", src);
                    continue;
                }
                rates.insert(src, v);
            }
            Err(err) => log::error!("failed to get rate: {err}, source: {}", src),
        }
    }
    rates
}

async fn collect(client: &Client, src: Source) -> anyhow::Result<Vec<Rate>> {
    let rates = match src {
        Source::Acba => collect_acba(&client).await?,
        Source::Ameria => collect_ameria(&client).await?,
        Source::Ardshin => collect_ardshin(&client).await?,
        Source::ArmSwiss => collect_arm_swiss(&client).await?,
        Source::CbAm => collect_cb_am(&client).await?,
        Source::Evoca => collect_evoca(&client).await?,
        Source::Fast => collect_fast(&client).await?,
        Source::Ineco => collect_ineco(&client).await?,
        Source::Mellat => collect_mellat(&client).await?,
        Source::Converse => collect_converse(&client).await?,
        Source::AEB => collect_aeb(&client).await?,
        Source::VtbAm => collect_vtb_am(&client).await?,
        Source::Artsakh => collect_artsakh(&client).await?,
        Source::UniBank => collect_unibank(&client).await?,
        Source::Amio => collect_amio(&client).await?,
        Source::Byblos => collect_byblos(&client).await?,
        Source::IdBank => collect_idbank(&client).await?,
        Source::MoEx => collect_moex(&client).await?,
        Source::Ararat => collect_ararat(&client).await?,
        Source::IdPay => collect_idpay(&client).await?,
        Source::Mir => collect_mir(&client).await?,
        Source::Sas => collect_sas(&client).await?,
        Source::Hsbc => collect_hsbc(&client).await?,
    };
    Ok(rates)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("no rates found")]
    NoRates,
}

async fn collect_acba(client: &Client) -> anyhow::Result<Vec<Rate>> {
    let resp: acba::Response = acba::Response::get_rates(&client).await?;
    let rates = parse_acba(resp)?;
    Ok(rates)
}

pub(crate) fn parse_acba(resp: acba::Response) -> anyhow::Result<Vec<Rate>> {
    let result = resp.result.ok_or(Error::NoRates)?;
    let mut results = vec![];
    for (rate_type, rates) in [
        (RateType::NoCash, result.rates.non_cash),
        (RateType::Cash, result.rates.cash),
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
    for rate in result.rates.cross {
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

async fn collect_ameria(client: &Client) -> anyhow::Result<Vec<Rate>> {
    let mut results = vec![];
    for rate_type in [RateType::NoCash, RateType::Cash] {
        let resp: armsoft::Response = ameria::Response::get_rates(&client, rate_type).await?;
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
            buy: Some(v.purchase),
            sell: Some(v.sale),
        })
        .collect()
}

async fn collect_ardshin(client: &Client) -> anyhow::Result<Vec<Rate>> {
    let resp: ardshin::Response = ardshin::Response::get_rates(&client).await?;
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

async fn collect_arm_swiss(client: &Client) -> anyhow::Result<Vec<Rate>> {
    let resp: arm_swiss::Response = arm_swiss::Response::get_rates(&client).await?;
    let lmasbrate = resp.lmasbrate.ok_or(Error::NoRates)?;
    let mut rates = vec![];
    for rate in lmasbrate {
        rates.push(Rate {
            from: rate.iso.clone(),
            to: Currency::default(),
            rate_type: RateType::NoCash,
            buy: Some(rate.bid),
            sell: Some(rate.offer),
        });
        rates.push(Rate {
            from: rate.iso.clone(),
            to: Currency::default(),
            rate_type: RateType::Cash,
            buy: Some(rate.bid_cash),
            sell: Some(rate.offer_cash),
        });
    }
    Ok(rates)
}

async fn collect_cb_am(client: &Client) -> anyhow::Result<Vec<Rate>> {
    let resp = cb_am::Response::get_rates(&client).await?;
    let rates = resp
        .soap_body
        .exchange_rates_latest_response
        .exchange_rates_latest_result
        .rates
        .exchange_rate
        .iter()
        .map(|v| Rate {
            from: v.iso.clone(),
            to: Currency::default(),
            rate_type: RateType::Cb,
            buy: Some(v.rate),
            sell: Some(v.rate),
        })
        .collect();
    Ok(rates)
}

async fn collect_evoca(client: &Client) -> anyhow::Result<Vec<Rate>> {
    let mut results = vec![];
    for rate_type in [RateType::NoCash, RateType::Cash] {
        let resp: armsoft::Response = evoca::Response::get_rates(&client, rate_type).await?;
        let rates = collect_armsoft(resp, rate_type);
        results.extend_from_slice(&rates);
    }
    Ok(results)
}

async fn collect_fast(client: &Client) -> anyhow::Result<Vec<Rate>> {
    let mut results = vec![];
    for rate_type in [RateType::NoCash, RateType::Cash] {
        let resp: fast::Response = fast::Response::get_rates(&client, rate_type).await?;
        let items = resp.rates.ok_or(Error::NoRates)?;
        let rates = items
            .iter()
            .map(|v| Rate {
                from: v.id.clone(),
                to: Currency::default(),
                rate_type,
                buy: Some(v.buy),
                sell: Some(v.sale),
            })
            .collect::<Vec<_>>();
        results.extend_from_slice(&rates);
    }
    Ok(results)
}

async fn collect_ineco(client: &Client) -> anyhow::Result<Vec<Rate>> {
    let resp: ineco::Response = ineco::Response::get_rates(&client).await?;
    if !resp.success {
        Err(Error::NoRates)?;
    }
    let items = resp.items.ok_or(Error::NoRates)?;
    let mut rates = vec![];
    for item in items {
        if item.cashless.buy.is_some() && item.cashless.sell.is_some() {
            rates.push(Rate {
                from: item.code.clone(),
                to: Currency::default(),
                rate_type: RateType::NoCash,
                buy: item.cashless.buy,
                sell: item.cashless.sell,
            });
        }
        if item.cash.buy.is_some() && item.cash.sell.is_some() {
            rates.push(Rate {
                from: item.code.clone(),
                to: Currency::default(),
                rate_type: RateType::Cash,
                buy: item.cash.buy,
                sell: item.cash.sell,
            });
        }
    }
    Ok(rates)
}

async fn collect_mellat(client: &Client) -> anyhow::Result<Vec<Rate>> {
    let resp: mellat::Response = mellat::Response::get_rates(&client).await?;
    let result = resp.result.ok_or(Error::NoRates)?;
    let mut rates = vec![];
    for rate in result.data {
        rates.push(Rate {
            from: rate.currency.clone(),
            to: Currency::default(),
            rate_type: RateType::NoCash,
            buy: Some(rate.buy),
            sell: Some(rate.sell),
        });
        rates.push(Rate {
            from: rate.currency.clone(),
            to: Currency::default(),
            rate_type: RateType::Cash,
            buy: Some(rate.buy_cash),
            sell: Some(rate.sell_cash),
        });
    }
    Ok(rates)
}

async fn collect_converse(client: &Client) -> anyhow::Result<Vec<Rate>> {
    let resp: converse::Response = converse::Response::get_rates(&client).await?;
    let mut results = vec![];
    for (rate_type, rates) in [
        (RateType::NoCash, resp.non_cash),
        (RateType::Cash, resp.cash),
    ] {
        let rates = rates
            .iter()
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

async fn collect_aeb(client: &Client) -> anyhow::Result<Vec<Rate>> {
    let resp: aeb::Response = aeb::Response::get_rates(&client).await?;
    let mut rates = vec![];
    for item in resp.rate_currency_settings {
        for rate in item.rates.iter().filter(|v| {
            [RateType::NoCash, RateType::Cash].contains(&v.rate_type)
                && v.buy_rate.is_some()
                && v.sell_rate.is_some()
        }) {
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

async fn collect_vtb_am(client: &Client) -> anyhow::Result<Vec<Rate>> {
    let resp: vtb_am::Response = vtb_am::Response::get_rates(&client).await?;
    let rates = resp
        .items
        .iter()
        .filter(|v| v.category_id == "internaltransfer")
        .flat_map(|v| v.rates.items.iter())
        .filter(|v| v.buy.is_some() && v.sell.is_some())
        .map(|v| Rate {
            from: v.base.currency.clone(),
            to: v.target.currency.clone(),
            rate_type: RateType::NoCash,
            buy: v.sell.as_ref().map(|v| v.min),
            sell: v.buy.as_ref().map(|v| v.max),
        })
        .collect();
    Ok(rates)
}

async fn collect_artsakh(client: &Client) -> anyhow::Result<Vec<Rate>> {
    let resp: lsoft::Response = artsakh::Response::get_rates(&client).await?;
    let rates = collect_lsoft(resp)?;
    Ok(rates)
}

async fn collect_unibank(client: &Client) -> anyhow::Result<Vec<Rate>> {
    let resp: lsoft::Response = unibank::Response::get_rates(&client).await?;
    let rates = collect_lsoft(resp)?;
    Ok(rates)
}

fn collect_lsoft(resp: lsoft::Response) -> anyhow::Result<Vec<Rate>> {
    let items = resp.get_currency_list.currency_list.ok_or(Error::NoRates)?;
    let mut rates = vec![];
    for item in items {
        if item.buy.is_some() && item.sell.is_some() {
            rates.push(Rate {
                from: item.external_id.clone(),
                to: Currency::default(),
                rate_type: RateType::NoCash,
                buy: item.buy,
                sell: item.sell,
            });
        }
        if item.csh_buy.is_some() && item.csh_sell.is_some() {
            rates.push(Rate {
                from: item.external_id.clone(),
                to: Currency::default(),
                rate_type: RateType::Cash,
                buy: item.csh_buy,
                sell: item.csh_sell,
            });
        }
    }
    Ok(rates)
}

async fn collect_amio(client: &Client) -> anyhow::Result<Vec<Rate>> {
    let mut results = vec![];
    for rate_type in [RateType::NoCash, RateType::Cash] {
        let resp: armsoft::Response = amio::Response::get_rates(&client, rate_type).await?;
        let rates = collect_armsoft(resp, rate_type);
        results.extend_from_slice(&rates);
    }
    Ok(results)
}

async fn collect_byblos(client: &Client) -> anyhow::Result<Vec<Rate>> {
    let mut results = vec![];
    for rate_type in [RateType::NoCash, RateType::Cash] {
        let resp: armsoft::Response = byblos::Response::get_rates(&client, rate_type).await?;
        let rates = collect_armsoft(resp, rate_type);
        results.extend_from_slice(&rates);
    }
    Ok(results)
}

async fn collect_idbank(client: &Client) -> anyhow::Result<Vec<Rate>> {
    let resp: idbank::Response = idbank::Response::get_rates(&client).await?;
    let result = resp.result.ok_or(Error::NoRates)?;
    let mut rates = vec![];
    for rate in result.currency_rate {
        if rate.buy.is_some() && rate.sell.is_some() {
            rates.push(Rate {
                from: rate.iso_txt.clone(),
                to: Currency::default(),
                rate_type: RateType::NoCash,
                buy: rate.buy,
                sell: rate.sell,
            });
        }
        if rate.csh_buy.is_some() && rate.csh_sell.is_some() {
            rates.push(Rate {
                from: rate.iso_txt.clone(),
                to: Currency::default(),
                rate_type: RateType::Cash,
                buy: rate.csh_buy,
                sell: rate.csh_sell,
            });
        }
    }
    Ok(rates)
}

async fn collect_moex(client: &Client) -> anyhow::Result<Vec<Rate>> {
    let resp: moex::Response = moex::Response::get_rates(&client).await?;
    const BOARD_ID: &str = "CETS";
    let facevalue = resp
        .securities
        .data
        .iter()
        .filter(|v| v.0 == BOARD_ID)
        .map(|v| v.1)
        .next()
        .unwrap_or(dec!(100.0));
    let last = resp
        .marketdata
        .data
        .iter()
        .filter(|v| v.0 == BOARD_ID)
        .filter_map(|v| v.1)
        .next();
    let Some(last) = last else {
        Err(Error::NoRates)?
    };
    let mut rates = vec![];
    if last == dec!(0.0) {
        return Ok(rates);
    }
    let rate = facevalue / last;
    rates.push(Rate {
        from: Currency::rub(),
        to: Currency::default(),
        rate_type: RateType::NoCash,
        buy: Some(rate),
        sell: Some(rate),
    });
    Ok(rates)
}

async fn collect_ararat(client: &Client) -> anyhow::Result<Vec<Rate>> {
    let mut results = vec![];
    for rate_type in [RateType::NoCash, RateType::Cash] {
        let resp: armsoft::Response = ararat::Response::get_rates(&client, rate_type).await?;
        let rates = collect_armsoft(resp, rate_type);
        results.extend_from_slice(&rates);
    }
    Ok(results)
}

async fn collect_idpay(client: &Client) -> anyhow::Result<Vec<Rate>> {
    let resp: idbank::Response = idbank::Response::get_rates(&client).await?;
    let result = resp.result.ok_or(Error::NoRates)?;
    const COMMISSION_RATE: Decimal = dec!(0.9);
    const RU_CARD_COMMISSION_RATE: Decimal = dec!(0.3);
    let rates = result
        .currency_rate
        .iter()
        .filter(|v| v.buy.is_some() && v.sell.is_some() && v.iso_txt == Currency::rub())
        .map(|v| {
            let buy = v.buy.unwrap();
            let sell = v.sell.unwrap();
            Rate {
                from: v.iso_txt.clone(),
                to: Currency::default(),
                rate_type: RateType::NoCash,
                buy: Some(buy - (COMMISSION_RATE / dec!(100.0) * buy)),
                sell: Some(
                    sell + ((COMMISSION_RATE + RU_CARD_COMMISSION_RATE) / dec!(100.0) * sell),
                ),
            }
        })
        .collect();
    Ok(rates)
}

async fn collect_mir(client: &Client) -> anyhow::Result<Vec<Rate>> {
    let resp: mir::Response = mir::Response::get_rates(&client).await?;
    let rates = resp
        .content
        .iter()
        .filter(|v| v.currency.strcode == Currency::default())
        .map(|v| {
            let buy = Some(dec!(1.0) / v.value_sell);
            let sell = Some(dec!(1.0) / v.value_buy);
            [
                Rate {
                    from: Currency::rub(),
                    to: Currency::default(),
                    rate_type: RateType::NoCash,
                    buy,
                    sell,
                },
                Rate {
                    from: Currency::rub(),
                    to: Currency::default(),
                    rate_type: RateType::Cash,
                    buy,
                    sell,
                },
            ]
        })
        .flatten()
        .collect();
    Ok(rates)
}

async fn collect_sas(client: &Client) -> anyhow::Result<Vec<Rate>> {
    let resp: sas::Response = sas::Response::get_rates(&client).await?;
    let rates = resp
        .rates
        .iter()
        .filter(|v| v.buy.is_some() && v.sell.is_some())
        .cloned()
        .collect();
    Ok(rates)
}

async fn collect_hsbc(client: &Client) -> anyhow::Result<Vec<Rate>> {
    let resp: hsbc::Response = hsbc::Response::get_rates(&client).await?;
    let rates = resp
        .rates
        .iter()
        .filter(|v| v.buy.is_some() && v.sell.is_some())
        .cloned()
        .collect();
    Ok(rates)
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

    #[tokio::test]
    async fn test_collect_acba() -> anyhow::Result<()> {
        let c = build_client()?;
        collect(&c, Source::Acba).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_ameria() -> anyhow::Result<()> {
        let c = build_client()?;
        collect(&c, Source::Ameria).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_ardshin() -> anyhow::Result<()> {
        let c = build_client()?;
        collect(&c, Source::Ardshin).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_arm_swiss() -> anyhow::Result<()> {
        let c = build_client()?;
        collect(&c, Source::ArmSwiss).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_cb_am() -> anyhow::Result<()> {
        let c = build_client()?;
        collect(&c, Source::CbAm).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_evoca() -> anyhow::Result<()> {
        let c = build_client()?;
        collect(&c, Source::Evoca).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_fast() -> anyhow::Result<()> {
        let c = build_client()?;
        collect(&c, Source::Fast).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_ineco() -> anyhow::Result<()> {
        let c = build_client()?;
        collect(&c, Source::Ineco).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_mellat() -> anyhow::Result<()> {
        let c = build_client()?;
        collect(&c, Source::Mellat).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_converse() -> anyhow::Result<()> {
        let c = build_client()?;
        collect(&c, Source::Converse).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_aeb() -> anyhow::Result<()> {
        let c = build_client()?;
        collect(&c, Source::AEB).await?;
        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_collect_vtb_am() -> anyhow::Result<()> {
        let c = build_client()?;
        collect(&c, Source::VtbAm).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_artsakh() -> anyhow::Result<()> {
        let c = build_client()?;
        collect(&c, Source::Artsakh).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_unibank() -> anyhow::Result<()> {
        let c = build_client()?;
        collect(&c, Source::UniBank).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_amio() -> anyhow::Result<()> {
        let c = build_client()?;
        collect(&c, Source::Amio).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_byblos() -> anyhow::Result<()> {
        let c = build_client()?;
        collect(&c, Source::Byblos).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_idbank() -> anyhow::Result<()> {
        let c = build_client()?;
        collect(&c, Source::IdBank).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_moex() -> anyhow::Result<()> {
        let c = build_client()?;
        collect(&c, Source::MoEx).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_ararat() -> anyhow::Result<()> {
        let c = build_client()?;
        collect(&c, Source::Ararat).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_idpay() -> anyhow::Result<()> {
        let c = build_client()?;
        collect(&c, Source::IdPay).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_mir() -> anyhow::Result<()> {
        let c = build_client()?;
        collect(&c, Source::Mir).await?;
        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_collect_sas() -> anyhow::Result<()> {
        let c = build_client()?;
        collect(&c, Source::Sas).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_hsbc() -> anyhow::Result<()> {
        let c = build_client()?;
        collect(&c, Source::Hsbc).await?;
        Ok(())
    }
}
