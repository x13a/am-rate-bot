use crate::sources::{Currency, Rate, RateType, Source};
use crate::DUNNO;
use rust_decimal::{Decimal, RoundingStrategy};
use rust_decimal_macros::dec;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

const RATE_DP: u32 = 4;
const DIFF_DP: u32 = 2;

#[derive(Debug)]
struct Edge {
    to: Currency,
    rate: Decimal,
}

fn build_graph(rates: &[Rate], rate_type: RateType) -> HashMap<Currency, Vec<Edge>> {
    let mut graph: HashMap<Currency, Vec<Edge>> = HashMap::new();
    let mut add_edge = |from: Currency, to: Currency, rate: Decimal| {
        graph.entry(from).or_default().push(Edge { to, rate });
    };
    for rate in rates
        .iter()
        .filter(|r| [rate_type, RateType::Cb].contains(&r.rate_type) && r.from != r.to)
    {
        if let Some(buy) = rate.buy {
            if buy > dec!(0.0) {
                add_edge(rate.from.clone(), rate.to.clone(), buy);
            }
        }
        if let Some(sell) = rate.sell {
            if sell > dec!(0.0) {
                add_edge(rate.to.clone(), rate.from.clone(), dec!(1.0) / sell);
            }
        }
    }
    graph
}

fn find_all_paths(
    graph: &HashMap<Currency, Vec<Edge>>,
    from: Currency,
    to: Currency,
) -> Vec<(Vec<Currency>, Decimal)> {
    assert_ne!(from, to);
    let mut paths = Vec::new();
    let mut path = Vec::new();
    let mut visited = HashSet::new();
    dfs(
        graph,
        from,
        to,
        &mut visited,
        &mut path,
        &mut paths,
        dec!(1.0),
    );
    paths
}

fn dfs(
    graph: &HashMap<Currency, Vec<Edge>>,
    from: Currency,
    to: Currency,
    visited: &mut HashSet<Currency>,
    path: &mut Vec<Currency>,
    paths: &mut Vec<(Vec<Currency>, Decimal)>,
    cumulative_rate: Decimal,
) {
    visited.insert(from.clone());
    path.push(from.clone());
    if from == to {
        paths.push((path.clone(), cumulative_rate));
    } else {
        if let Some(edges) = graph.get(&from) {
            for edge in edges {
                if !visited.contains(&edge.to) {
                    let new_cumulative_rate = cumulative_rate * edge.rate;
                    dfs(
                        graph,
                        edge.to.clone(),
                        to.clone(),
                        visited,
                        path,
                        paths,
                        new_cumulative_rate,
                    );
                }
            }
        }
    }
    path.pop();
    visited.remove(&from);
}

pub fn generate_from_to_table(
    from: Currency,
    to: Currency,
    rates: &HashMap<Source, Vec<Rate>>,
    rate_type: RateType,
    is_inv: bool,
) -> String {
    if from.is_empty() || to.is_empty() {
        return DUNNO.into();
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
    let sort = if is_inv {
        |a: Decimal, b: Decimal| a.partial_cmp(&b).expect("panic")
    } else {
        |a: Decimal, b: Decimal| b.partial_cmp(&a).expect("panic")
    };
    for (src, rates) in rates {
        let graph = build_graph(&rates, rate_type);
        let mut paths = find_all_paths(&graph, from.clone(), to.clone());
        if is_inv {
            paths.iter_mut().for_each(|v| v.1 = dec!(1.0) / v.1);
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
            let rate_str = decimal_to_string(rate, RATE_DP);
            rate_width = rate_width.max(rate_str.len());
            table.push(Row {
                src: src.clone(),
                rate,
                rate_str,
                diff: dec!(0.0),
                diff_str: "".into(),
                path: path.clone(),
            });
        }
    }
    table.sort_by(|a, b| match sort(a.rate, b.rate) {
        std::cmp::Ordering::Equal => {
            let a_src = a.src.to_string();
            let b_src = b.src.to_string();
            a_src.cmp(&b_src)
        }
        other => other,
    });
    let best_rate = table
        .iter()
        .filter(|r| !Source::get_not_banks().contains(&r.src))
        .map(|r| r.rate)
        .next()
        .unwrap_or_default();
    let mut is_desc = false;
    let mut rate = dec!(0.0);
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
    table.iter_mut().for_each(|row| {
        row.diff = ((best_rate - row.rate) / row.rate) * dec!(100.0);
        if is_desc && !row.diff.is_zero() {
            row.diff = -row.diff;
        }
        row.diff_str = decimal_to_string(row.diff, DIFF_DP);
        diff_width = diff_width.max(row.diff_str.len());
    });
    let mut s = String::new();
    for row in table {
        writeln!(
            &mut s,
            "{} {:<src_width$} | {:<rate_width$} | {:>diff_width$} | {}",
            row.src.prefix(),
            row.src.to_string(),
            row.rate_str,
            row.diff_str,
            row.path
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join("/"),
        )
        .expect("panic");
    }
    if s.is_empty() {
        DUNNO.into()
    } else {
        s
    }
}

fn decimal_to_string(value: Decimal, dp: u32) -> String {
    value
        .round_dp_with_strategy(dp, RoundingStrategy::MidpointAwayFromZero)
        .normalize()
        .to_string()
}

pub fn generate_src_table(
    src: Source,
    rates: &HashMap<Source, Vec<Rate>>,
    mut rate_type: RateType,
) -> String {
    let Some(rates) = rates.get(&src) else {
        return DUNNO.into();
    };

    #[derive(Debug)]
    struct Row {
        buy_str: String,
        sell_str: String,
        from: Currency,
        to: Currency,
    }

    if src == Source::CbAm {
        rate_type = RateType::Cb;
    }
    let mut table = vec![];
    let mut buy_width: usize = 0;
    let mut sell_width: usize = 0;
    const NO_RATE: &str = "-";
    for rate in rates
        .iter()
        .filter(|v| v.rate_type == rate_type && v.from != v.to)
    {
        let buy_str = match rate.buy {
            Some(buy) => decimal_to_string(buy, RATE_DP),
            _ => NO_RATE.to_string(),
        };
        let sell_str = match rate.sell {
            Some(sell) => decimal_to_string(sell, RATE_DP),
            _ => NO_RATE.to_string(),
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
        .expect("panic");
    }
    if s.is_empty() {
        DUNNO.into()
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collector::{collect_all, filter_collection, parse_acba};
    use crate::sources::{acba, Config};
    use reqwest::Client;
    use std::time::Duration;
    use strum::IntoEnumIterator;

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

    const ACBA_DATA: &str = r#"{
      "Description": null,
      "ResultCode": 1,
      "Result": {
        "rates": {
          "last_update_date": "2024-09-16T16:07:00+04:00",
          "cash": [
            {
              "Buy": "384.5",
              "Sell": "390.5",
              "CB": "387.19",
              "Currency": "USD"
            },
            {
              "Buy": "425",
              "Sell": "437",
              "CB": "430.63",
              "Currency": "EUR"
            },
            {
              "Buy": "4.21",
              "Sell": "4.44",
              "CB": "4.25",
              "Currency": "RUR"
            },
            {
              "Buy": "499",
              "Sell": "519",
              "CB": "510.82",
              "Currency": "GBP"
            },
            {
              "Buy": "448",
              "Sell": "468",
              "CB": "458.38",
              "Currency": "CHF"
            },
            {
              "Buy": "140",
              "Sell": "160",
              "CB": "143.75",
              "Currency": "GEL"
            }
          ],
          "non_cash": [
            {
              "Buy": "384",
              "Sell": "390",
              "CB": "387.19",
              "Currency": "USD"
            },
            {
              "Buy": "425",
              "Sell": "437",
              "CB": "430.63",
              "Currency": "EUR"
            },
            {
              "Buy": "4.21",
              "Sell": "4.44",
              "CB": "4.25",
              "Currency": "RUR"
            },
            {
              "Buy": "499",
              "Sell": "519",
              "CB": "510.82",
              "Currency": "GBP"
            },
            {
              "Buy": "448",
              "Sell": "468",
              "CB": "458.38",
              "Currency": "CHF"
            },
            {
              "Buy": "140",
              "Sell": "160",
              "CB": "143.75",
              "Currency": "GEL"
            }
          ],
          "card": [
            {
              "Buy": "384",
              "Sell": "390.5",
              "CB": "387.19",
              "Currency": "USD"
            },
            {
              "Buy": "425",
              "Sell": "437",
              "CB": "430.63",
              "Currency": "EUR"
            },
            {
              "Buy": "4.21",
              "Sell": "4.44",
              "CB": "4.25",
              "Currency": "RUR"
            },
            {
              "Buy": "499",
              "Sell": "519",
              "CB": "510.82",
              "Currency": "GBP"
            },
            {
              "Buy": "448",
              "Sell": "468",
              "CB": "458.38",
              "Currency": "CHF"
            },
            {
              "Buy": "140",
              "Sell": "160",
              "CB": "143.75",
              "Currency": "GEL"
            }
          ],
          "cross": [
            {
              "Buy": "1.0883483",
              "Sell": "1.1380208",
              "Currency": "EUR/USD"
            },
            {
              "Buy": "86.4864865",
              "Sell": "92.7553444",
              "Currency": "USD/RUR"
            },
            {
              "Buy": "1.2778489",
              "Sell": "1.3515625",
              "Currency": "GBP/USD"
            },
            {
              "Buy": "0.8205128",
              "Sell": "0.8716518",
              "Currency": "USD/CHF"
            },
            {
              "Buy": "95.7207207",
              "Sell": "103.8004751",
              "Currency": "EUR/RUR"
            },
            {
              "Buy": "0.9081197",
              "Sell": "0.9754464",
              "Currency": "EUR/CHF"
            },
            {
              "Buy": "0.8188825",
              "Sell": "0.8757515",
              "Currency": "EUR/GBP"
            }
          ],
          "currencies": [
            {
              "Key": "AMD",
              "Value": "AMD"
            },
            {
              "Key": "USD",
              "Value": "USD"
            },
            {
              "Key": "EUR",
              "Value": "EUR"
            },
            {
              "Key": "RUR",
              "Value": "RUR"
            },
            {
              "Key": "GBP",
              "Value": "GBP"
            },
            {
              "Key": "CHF",
              "Value": "CHF"
            },
            {
              "Key": "GEL",
              "Value": "GEL"
            }
          ]
        }
      },
      "ResultCodeDescription": "normal"
    }"#;

    fn get_test_cases() -> Vec<(Currency, Currency)> {
        vec![
            (Currency::usd(), Currency::rub()),
            (Currency::eur(), Currency::rub()),
            (Currency::eur(), Currency::usd()),
            (Currency::usd(), Currency::default()),
            (Currency::rub(), Currency::default()),
        ]
    }

    #[test]
    fn test_graph() -> anyhow::Result<()> {
        let acba: acba::Response = serde_json::from_str(ACBA_DATA)?;
        let rates = parse_acba(acba)?;
        let graph = build_graph(&rates, RateType::NoCash);
        let test_cases = get_test_cases();
        for (from, to) in test_cases {
            let mut paths = find_all_paths(&graph, from.clone(), to.clone());
            paths.sort_by(|a, b| b.1.partial_cmp(&a.1).expect("panic"));
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_generate_from_to_table() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_src_config()?;
        let results = collect_all(&client, &config).await;
        let rates = filter_collection(results);
        let test_cases = get_test_cases();
        for (from, to) in test_cases {
            let _ =
                generate_from_to_table(from.clone(), to.clone(), &rates, RateType::NoCash, false);
            let _ =
                generate_from_to_table(to.clone(), from.clone(), &rates, RateType::NoCash, true);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_generate_src_table() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_src_config()?;
        let results = collect_all(&client, &config).await;
        let rates = filter_collection(results);
        for src in Source::iter() {
            let _ = generate_src_table(src, &rates, RateType::NoCash);
        }
        Ok(())
    }
}
