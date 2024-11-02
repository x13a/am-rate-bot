use crate::{
    graph,
    source::{self, Config, Rate, RateType, Source},
};
use rust_decimal::Decimal;
use std::collections::HashMap;
use strum::{EnumCount, IntoEnumIterator};
use tokio::sync::mpsc;

pub async fn collect_all(
    client: &reqwest::Client,
    config: &Config,
) -> HashMap<Source, anyhow::Result<Vec<Rate>>> {
    let mut results = HashMap::new();
    let (tx, mut rx) = mpsc::channel(Source::COUNT);
    for src in Source::iter().filter(|v| config.is_enabled_for(*v)) {
        let client = client.clone();
        let config = config.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            let result = source::collect(&client, &config, src).await;
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
                            && (v.buy.is_some_and(|v| v > Decimal::ZERO)
                                || v.sell.is_some_and(|v| v > Decimal::ZERO))
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                if graph::detect_arbitrage(&v, RateType::NoCash) {
                    log::info!("arbitrage detected: {src}");
                }
                rates.insert(src, v);
            }
            Err(err) => log::error!("src: {src}, err: {err}"),
        }
    }
    rates
}
