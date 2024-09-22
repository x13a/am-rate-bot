use crate::sources;
use crate::sources::{
    acba, aeb, ameria, ardshin, arm_swiss, artsakh, cba, converse, evoca, fast, ineco, mellat, vtb,
    Currency, RateType, Source, SourceCashUrlTrait, SourceSingleUrlTrait,
};
use reqwest::Client;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct Rate {
    pub(crate) currency: Currency,
    pub(crate) rate_type: RateType,
    pub(crate) buy: f64,
    pub(crate) sell: f64,
}

pub async fn collect_all(client: &Client) -> HashMap<Source, Result<Vec<Rate>, Error>> {
    let mut results = HashMap::new();
    let (tx, mut rx) = mpsc::channel(Source::iter().count());
    for source in Source::iter() {
        let client = client.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            let result = collect(&client, source).await;
            tx.send((source, result)).await.expect("panic");
        });
    }
    drop(tx);
    while let Some(result) = rx.recv().await {
        results.insert(result.0, result.1);
    }
    results
}

pub fn filter_collection(
    results: HashMap<Source, Result<Vec<Rate>, Error>>,
) -> HashMap<Source, Vec<Rate>> {
    let mut rates = HashMap::new();
    for (source, result) in results {
        match result {
            Ok(v) => {
                if v.is_empty() {
                    log::warn!("no rate found, source: {}", source);
                    continue;
                }
                rates.insert(source, v);
            }
            Err(err) => log::error!("failed to get rate: {err}, source: {}", source),
        }
    }
    rates
}

async fn collect(client: &Client, source: Source) -> Result<Vec<Rate>, Error> {
    let rates = match source {
        Source::Acba => collect_acba(&client).await?,
        Source::Ameria => collect_ameria(&client).await?,
        Source::Ardshin => collect_ardshin(&client).await?,
        Source::ArmSwiss => collect_arm_swiss(&client).await?,
        Source::CBA => collect_cba(&client).await?,
        Source::Evoca => collect_evoca(&client).await?,
        Source::Fast => collect_fast(&client).await?,
        Source::Ineco => collect_ineco(&client).await?,
        Source::Mellat => collect_mellat(&client).await?,
        Source::Converse => collect_converse(&client).await?,
        Source::AEB => collect_aeb(&client).await?,
        Source::VTB => collect_vtb(&client).await?,
        Source::Artsakh => collect_artsakh(&client).await?,
    };
    Ok(rates)
}

#[derive(Debug)]
pub enum Error {
    Sources(sources::Error),
    NoRates,
}

impl From<sources::Error> for Error {
    fn from(err: sources::Error) -> Self {
        Self::Sources(err)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

async fn collect_acba(client: &Client) -> Result<Vec<Rate>, Error> {
    let acba: acba::Response = acba::Response::get_rates(&client).await?;
    let rates = parse_acba(acba)?;
    Ok(rates)
}

pub(crate) fn parse_acba(acba: acba::Response) -> Result<Vec<Rate>, Error> {
    if acba.result_code != 1 {
        return Err(Error::NoRates);
    }
    let result = acba.result.ok_or(Error::NoRates)?;
    let rates = result
        .rates
        .non_cash
        .iter()
        .map(|v| Rate {
            currency: v.currency.clone(),
            rate_type: RateType::NoCash,
            buy: v.buy,
            sell: v.sell,
        })
        .chain(result.rates.cross.iter().map(|v| Rate {
            currency: v.currency.clone(),
            rate_type: RateType::Cross,
            buy: v.buy,
            sell: v.sell,
        }))
        .collect();
    Ok(rates)
}

async fn collect_ameria(client: &Client) -> Result<Vec<Rate>, Error> {
    let ameria: ameria::Response = ameria::Response::get_rates(&client, RateType::NoCash).await?;
    let rates = ameria
        .array_of_exchange_rate
        .iter()
        .map(|v| Rate {
            currency: v.currency.clone(),
            rate_type: RateType::NoCash,
            buy: v.purchase,
            sell: v.sale,
        })
        .collect();
    Ok(rates)
}

async fn collect_ardshin(client: &Client) -> Result<Vec<Rate>, Error> {
    let ardshin: ardshin::Response = ardshin::Response::get_rates(&client).await?;
    let rates = ardshin
        .data
        .currencies
        .no_cash
        .iter()
        .map(|v| Rate {
            currency: v.curr_type.clone(),
            rate_type: RateType::NoCash,
            buy: v.buy,
            sell: v.sell,
        })
        .collect();
    Ok(rates)
}

async fn collect_arm_swiss(client: &Client) -> Result<Vec<Rate>, Error> {
    let arm_swiss: arm_swiss::Response = arm_swiss::Response::get_rates(&client).await?;
    let rates = arm_swiss
        .lmasbrate
        .iter()
        .map(|v| Rate {
            currency: v.iso.clone(),
            rate_type: RateType::NoCash,
            buy: v.bid,
            sell: v.offer,
        })
        .collect();
    Ok(rates)
}

async fn collect_cba(client: &Client) -> Result<Vec<Rate>, Error> {
    let cba = cba::Response::get_rates(&client).await?;
    let rates = cba
        .soap_body
        .exchange_rates_latest_response
        .exchange_rates_latest_result
        .rates
        .exchange_rate
        .iter()
        .map(|v| Rate {
            currency: v.iso.clone(),
            rate_type: RateType::CB,
            buy: v.rate,
            sell: v.rate,
        })
        .collect();
    Ok(rates)
}

async fn collect_evoca(client: &Client) -> Result<Vec<Rate>, Error> {
    let evoca: evoca::Response = evoca::Response::get_rates(&client, RateType::NoCash).await?;
    let rates = evoca
        .array_of_exchange_rate
        .iter()
        .map(|v| Rate {
            currency: v.currency.clone(),
            rate_type: RateType::NoCash,
            buy: v.purchase,
            sell: v.sale,
        })
        .collect();
    Ok(rates)
}

async fn collect_fast(client: &Client) -> Result<Vec<Rate>, Error> {
    let fast: fast::Response = fast::Response::get_rates(&client, RateType::NoCash).await?;
    let result = fast.rates.ok_or(Error::NoRates)?;
    let rates = result
        .iter()
        .map(|v| Rate {
            currency: v.id.clone(),
            rate_type: RateType::NoCash,
            buy: v.buy,
            sell: v.sale,
        })
        .collect();
    Ok(rates)
}

async fn collect_ineco(client: &Client) -> Result<Vec<Rate>, Error> {
    let ineco: ineco::Response = ineco::Response::get_rates(&client).await?;
    if !ineco.success {
        return Err(Error::NoRates);
    }
    let result = ineco.items.ok_or(Error::NoRates)?;
    let rates = result
        .iter()
        .filter(|v| v.cashless.buy.is_some() && v.cashless.sell.is_some())
        .map(|v| Rate {
            currency: v.code.clone(),
            rate_type: RateType::NoCash,
            buy: v.cashless.buy.unwrap(),
            sell: v.cashless.sell.unwrap(),
        })
        .collect();
    Ok(rates)
}

async fn collect_mellat(client: &Client) -> Result<Vec<Rate>, Error> {
    let mellat: mellat::Response = mellat::Response::get_rates(&client).await?;
    let result = mellat.result.ok_or(Error::NoRates)?;
    let rates = result
        .data
        .iter()
        .map(|v| Rate {
            currency: v.currency.clone(),
            rate_type: RateType::NoCash,
            buy: v.buy,
            sell: v.sell,
        })
        .collect();
    Ok(rates)
}

async fn collect_converse(client: &Client) -> Result<Vec<Rate>, Error> {
    let converse: converse::Response = converse::Response::get_rates(&client).await?;
    let rates = converse
        .non_cash
        .iter()
        .map(|v| Rate {
            currency: v.currency.iso.clone(),
            rate_type: RateType::NoCash,
            buy: v.buy,
            sell: v.sell,
        })
        .collect();
    Ok(rates)
}

async fn collect_aeb(client: &Client) -> Result<Vec<Rate>, Error> {
    let aeb: aeb::Response = aeb::Response::get_rates(&client).await?;
    let mut rates = vec![];
    for item in aeb.rate_currency_settings {
        for rate in item.rates.iter().filter(|v| {
            v.rate_type == RateType::NoCash && v.buy_rate.is_some() && v.sell_rate.is_some()
        }) {
            rates.push(Rate {
                currency: item.currency_code.clone(),
                rate_type: RateType::NoCash,
                buy: rate.buy_rate.unwrap(),
                sell: rate.sell_rate.unwrap(),
            });
        }
    }
    Ok(rates)
}

async fn collect_vtb(client: &Client) -> Result<Vec<Rate>, Error> {
    let vtb: vtb::Response = vtb::Response::get_rates(&client).await?;
    let mut rates = vec![];
    for item in vtb
        .items
        .iter()
        .filter(|v| v.category_id == "internaltransfer")
    {
        for item in item.rates.items.iter().filter(|v| v.sell.is_some()) {
            rates.push(Rate {
                currency: item.base.currency.clone(),
                rate_type: RateType::NoCash,
                buy: item.sell.as_ref().unwrap().min,
                sell: item.buy.min,
            });
        }
    }
    Ok(rates)
}

async fn collect_artsakh(client: &Client) -> Result<Vec<Rate>, Error> {
    let resp: artsakh::Response = artsakh::Response::get_rates(&client).await?;
    let items = resp
        .get_currency_list
        .currency_list
        .ok_or(Error::NoRates)?;
    let rates = items
        .iter()
        .filter(|v| v.sell.is_some() && v.buy.is_some())
        .map(|v| Rate {
            currency: v.external_id.clone(),
            rate_type: RateType::NoCash,
            buy: v.buy.unwrap(),
            sell: v.sell.unwrap(),
        })
        .collect();
    Ok(rates)
}

mod tests {
    use super::*;
    use crate::sources::tests::build_client;

    #[tokio::test]
    async fn test_collect_acba() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        collect(&c, Source::Acba).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_ameria() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        collect(&c, Source::Ameria).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_ardshin() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        collect(&c, Source::Ardshin).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_arm_swiss() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        collect(&c, Source::ArmSwiss).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_cba() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        collect(&c, Source::CBA).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_evoca() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        collect(&c, Source::Evoca).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_fast() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        collect(&c, Source::Fast).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_ineco() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        collect(&c, Source::Ineco).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_mellat() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        collect(&c, Source::Mellat).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_converse() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        collect(&c, Source::Converse).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_aeb() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        collect(&c, Source::AEB).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_vtb() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        collect(&c, Source::VTB).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_artsakh() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        collect(&c, Source::Artsakh).await?;
        Ok(())
    }
}
