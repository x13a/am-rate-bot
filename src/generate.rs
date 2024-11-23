use crate::{
    config,
    graph,
    source::{Currency, Rate, RateType, Source},
};
use rust_decimal::{Decimal, RoundingStrategy};
use std::{collections::HashMap, fmt::Write};

pub fn conv_table(
    from: Currency,
    to: Currency,
    rates: &HashMap<Source, Vec<Rate>>,
    rate_type: RateType,
    inv: bool,
    cfg: &config::Gen,
) -> String {
    if from.is_empty() || to.is_empty() {
        return "".into();
    }

    #[derive(Debug)]
    struct Row {
        src: Source,
        rate: Decimal,
        rate_str: String,
        diff: Decimal,
        diff_str: String,
        path: Vec<Currency>,
    }

    let mut table = vec![];
    let mut src_width: usize = 0;
    let mut rate_width: usize = 0;
    let sort = if inv {
        |a: Decimal, b: Decimal| a.partial_cmp(&b).expect("panic")
    } else {
        |a: Decimal, b: Decimal| b.partial_cmp(&a).expect("panic")
    };
    'outer: for (src, rates) in rates {
        let graph = graph::build(&rates, rate_type);
        let mut paths = graph::find_all_paths(&graph, from.clone(), to.clone());
        if paths.is_empty() {
            continue;
        }
        if inv {
            for path in paths.iter_mut() {
                if path.1.is_zero() {
                    continue 'outer;
                }
                path.1 = Decimal::ONE / path.1;
            }
        }
        paths.sort_by(|a, b| sort(a.1, b.1));
        if src.is_bank() {
            let max_len = paths.iter().map(|v| v.0.len()).max().unwrap_or(3);
            for i in 2..max_len + 1 {
                let pos = paths.iter().position(|v| v.0.len() == i);
                if let Some(pos) = pos {
                    paths.drain(pos + 1..);
                    break;
                }
            }
        }
        src_width = src_width.max(src.to_string().len());
        for (path, rate) in paths {
            let rate_str = decimal_to_string(rate, cfg.rate_dp);
            rate_width = rate_width.max(rate_str.len());
            table.push(Row {
                src: src.clone(),
                rate,
                rate_str,
                diff: Decimal::ZERO,
                diff_str: "".into(),
                path: path.clone(),
            });
        }
    }
    table.sort_by(|a, b| match sort(a.rate, b.rate) {
        std::cmp::Ordering::Equal => a.src.cmp(&b.src),
        other => other,
    });
    let best_rate = table
        .iter()
        .filter(|r| r.src.is_bank())
        .map(|r| r.rate)
        .next()
        .unwrap_or_default();
    let mut is_desc = false;
    let mut rate = Decimal::ZERO;
    for (idx, row) in table.iter().enumerate() {
        if idx == 0 {
            rate = row.rate;
            continue;
        }
        if rate < row.rate {
            break;
        } else if rate > row.rate {
            is_desc = true;
            break;
        }
    }
    let mut diff_width: usize = 0;
    for row in table.iter_mut() {
        if row.rate.is_zero() {
            continue;
        }
        row.diff = ((best_rate - row.rate) / row.rate) * Decimal::ONE_HUNDRED;
        if is_desc && !row.diff.is_zero() {
            row.diff = -row.diff;
        }
        row.diff_str = decimal_to_string(row.diff, cfg.diff_dp);
        diff_width = diff_width.max(row.diff_str.len());
    }
    let mut s = String::new();
    for row in table {
        writeln!(
            &mut s,
            "{} {:<src_width$} | {:<rate_width$} | {:>diff_width$} | {}",
            row.src.prefix(),
            row.src,
            row.rate_str,
            row.diff_str,
            row.path
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join("/"),
        )
        .unwrap();
    }
    s
}

fn decimal_to_string(value: Decimal, dp: u32) -> String {
    value
        .round_dp_with_strategy(dp, RoundingStrategy::MidpointAwayFromZero)
        .normalize()
        .to_string()
}

pub fn src_table(
    src: Source,
    rates: &HashMap<Source, Vec<Rate>>,
    rate_type: RateType,
    cfg: &config::Gen,
) -> String {
    let Some(rates) = rates.get(&src) else {
        return "".into();
    };

    #[derive(Debug)]
    struct Row {
        buy_str: String,
        sell_str: String,
        from: Currency,
        to: Currency,
    }

    let mut rate_type = rate_type;
    if src == Source::Cb {
        rate_type = RateType::Cb;
    }
    let mut table = vec![];
    let mut buy_width: usize = 0;
    let mut sell_width: usize = 0;
    const NO_RATE: &str = "-";
    for rate in rates.iter().filter(|v| v.rate_type == rate_type) {
        let buy_str = match rate.buy {
            Some(buy) => decimal_to_string(buy, cfg.rate_dp),
            _ => NO_RATE.into(),
        };
        let sell_str = match rate.sell {
            Some(sell) => decimal_to_string(sell, cfg.rate_dp),
            _ => NO_RATE.into(),
        };
        let row = Row {
            buy_str,
            sell_str,
            from: rate.from.clone(),
            to: rate.to.clone(),
        };
        buy_width = buy_width.max(row.buy_str.len());
        sell_width = sell_width.max(row.sell_str.len());
        table.push(row);
    }
    let mut s = String::new();
    for row in table {
        writeln!(
            &mut s,
            "{:<buy_width$} | {:<sell_width$} | {}/{}",
            row.buy_str, row.sell_str, row.from, row.to,
        )
        .unwrap();
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{collector, config::Config};
    use std::{sync::LazyLock, time::Duration};
    use strum::{EnumCount, IntoEnumIterator};
    use tokio::sync::mpsc;

    static CFG: LazyLock<Config> =
        LazyLock::new(|| toml::from_str(include_str!("../config/config.toml")).unwrap());

    fn build_client(cfg: &Config) -> reqwest::Result<reqwest::Client> {
        reqwest::ClientBuilder::new()
            .timeout(Duration::from_secs(cfg.bot.reqwest_timeout))
            .build()
    }

    fn get_conversations() -> Vec<(Currency, Currency)> {
        vec![
            (Currency::default(), Currency::usd()),
            (Currency::default(), Currency::eur()),
            (Currency::rub(), Currency::default()),
            (Currency::rub(), Currency::usd()),
            (Currency::rub(), Currency::eur()),
            (Currency::usd(), Currency::eur()),
        ]
    }

    async fn collect() -> anyhow::Result<HashMap<Source, Vec<Rate>>> {
        let client = build_client(&CFG)?;
        let mut result = HashMap::new();
        let (tx, mut rx) = mpsc::channel(Source::COUNT);
        let client = client.clone();
        let cfg = CFG.clone();
        {
            let tx = tx.clone();
            tokio::spawn(async move {
                collector::collect(&client, &cfg.src, tx).await;
            });
        }
        drop(tx);
        while let Some((src, rates)) = rx.recv().await {
            result.insert(src, rates);
        }
        Ok(result)
    }

    #[tokio::test]
    async fn test_conv_table() -> anyhow::Result<()> {
        let rates = collect().await?;
        for (from, to) in get_conversations() {
            let _ = conv_table(
                from.clone(),
                to.clone(),
                &rates,
                RateType::NoCash,
                false,
                &CFG.gen,
            );
            let _ = conv_table(
                to.clone(),
                from.clone(),
                &rates,
                RateType::NoCash,
                true,
                &CFG.gen,
            );
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_src_table() -> anyhow::Result<()> {
        let rates = collect().await?;
        for src in Source::iter() {
            let _ = src_table(src, &rates, RateType::NoCash, &CFG.gen);
        }
        Ok(())
    }
}
