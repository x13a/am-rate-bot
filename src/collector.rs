use crate::{
    graph,
    source::{self, Config, Rate, RateType, Source},
};
use rust_decimal::Decimal;
use strum::IntoEnumIterator;
use tokio::sync::mpsc;

pub async fn collect(
    client: &reqwest::Client,
    cfg: &Config,
    tx: mpsc::Sender<(Source, Vec<Rate>)>,
) {
    for src in Source::iter().filter(|v| cfg.is_enabled_for(*v)) {
        let client = client.clone();
        let cfg = cfg.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            let result = source::collect(&client, &cfg, src).await;
            match result {
                Ok(rates) => {
                    let rates = rates
                        .iter()
                        .filter(|v| {
                            (!v.from.is_empty() && !v.to.is_empty())
                                && (v.buy.is_some_and(|v| v > Decimal::ZERO)
                                    || v.sell.is_some_and(|v| v > Decimal::ZERO))
                        })
                        .cloned()
                        .collect::<Vec<_>>();
                    if rates.is_empty() {
                        return;
                    }
                    if graph::detect_arbitrage(&rates, RateType::NoCash) {
                        log::info!("arbitrage detected: {src}");
                    }
                    tx.send((src, rates)).await.unwrap();
                }
                Err(err) => log::error!("src: {src}, err: {err}"),
            }
        });
    }
}
