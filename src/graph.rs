use crate::source::{Currency, Rate, RateType};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct Edge {
    to: Currency,
    rate: Decimal,
}

pub fn build(rates: &[Rate], rate_type: RateType) -> HashMap<Currency, Vec<Edge>> {
    let mut graph: HashMap<Currency, Vec<Edge>> = HashMap::new();
    let mut add_edge = |from: Currency, to: Currency, rate: Decimal| {
        graph.entry(from).or_default().push(Edge { to, rate });
    };
    for rate in rates
        .iter()
        .filter(|r| [rate_type, RateType::Cb].contains(&r.rate_type))
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

pub fn find_all_paths(
    graph: &HashMap<Currency, Vec<Edge>>,
    from: Currency,
    to: Currency,
) -> Vec<(Vec<Currency>, Decimal)> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::acba;

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

    #[test]
    fn test_graph() -> anyhow::Result<()> {
        let resp: acba::Response = serde_json::from_str(ACBA_DATA)?;
        let rates = acba::parse(resp)?;
        let graph = build(&rates, RateType::NoCash);
        for (from, to) in get_conversations() {
            let _ = find_all_paths(&graph, from.clone(), to.clone());
        }
        Ok(())
    }
}
