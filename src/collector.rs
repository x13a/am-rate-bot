#[cfg(feature = "moex")]
use crate::source::moex;
use crate::{
    config::Config,
    graph,
    source::{self, Rate, RateType, Source},
};
use rust_decimal::Decimal;
use std::sync::Arc;
#[cfg(feature = "moex")]
use std::{env, sync::LazyLock};
use strum::IntoEnumIterator;
use tokio::sync::mpsc;

pub async fn collect(
    client: &reqwest::Client,
    cfg: Arc<Config>,
    tx: mpsc::Sender<(Source, Vec<Rate>)>,
) {
    #[cfg(feature = "moex")]
    static MOEX_OK: LazyLock<bool> = LazyLock::new(|| {
        !env::var(moex::ENV_TINKOFF_TOKEN)
            .unwrap_or_default()
            .is_empty()
    });
    for src in Source::iter().filter(|v| cfg.src.is_enabled_for(*v)) {
        #[cfg(feature = "moex")]
        if src == Source::MOEX && !MOEX_OK.clone() {
            continue;
        }
        let client = client.clone();
        let cfg = cfg.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            let result = source::collect(&client, &cfg.src, src).await;
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
