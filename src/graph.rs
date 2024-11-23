use crate::source::{Currency, Rate, RateType};
use rust_decimal::{prelude::ToPrimitive, Decimal};
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
            if buy > Decimal::ZERO {
                add_edge(rate.from.clone(), rate.to.clone(), buy);
            }
        }
        if let Some(sell) = rate.sell {
            if sell > Decimal::ZERO {
                add_edge(rate.to.clone(), rate.from.clone(), Decimal::ONE / sell);
            }
        }
    }
    graph
}

pub fn find_all_paths(
    graph: &HashMap<Currency, Vec<Edge>>,
    from: &Currency,
    to: &Currency,
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
        Decimal::ONE,
    );
    paths
}

fn dfs(
    graph: &HashMap<Currency, Vec<Edge>>,
    from: &Currency,
    to: &Currency,
    visited: &mut HashSet<Currency>,
    path: &mut Vec<Currency>,
    paths: &mut Vec<(Vec<Currency>, Decimal)>,
    rate: Decimal,
) {
    visited.insert(from.clone());
    path.push(from.clone());
    if from == to {
        paths.push((path.clone(), rate));
    } else {
        if let Some(edges) = graph.get(&from) {
            for edge in edges {
                if visited.contains(&edge.to) {
                    continue;
                }
                let new_rate = rate * edge.rate;
                dfs(graph, &edge.to, to, visited, path, paths, new_rate);
            }
        }
    }
    path.pop();
    visited.remove(&from);
}

pub fn detect_arbitrage(rates: &[Rate], rate_type: RateType) -> bool {
    let rates = rates
        .iter()
        .filter(|v| v.rate_type == rate_type)
        .cloned()
        .collect::<Vec<_>>();
    let currency_indices = {
        let mut map = HashMap::new();
        let mut idx = 0;
        for rate in &rates {
            if !map.contains_key(&rate.from) {
                map.insert(rate.from.clone(), idx);
                idx += 1;
            }
            if !map.contains_key(&rate.to) {
                map.insert(rate.to.clone(), idx);
                idx += 1;
            }
        }
        map
    };
    if currency_indices.is_empty() {
        return false;
    }
    let num_currencies = currency_indices.len();
    let mut dist = vec![f64::INFINITY; num_currencies];
    dist[0] = 0.0;
    let mut edges = Vec::new();
    for rate in &rates {
        let from = currency_indices.get(&rate.from).unwrap();
        let to = currency_indices.get(&rate.to).unwrap();
        if let Some(buy) = rate.buy {
            if buy > Decimal::ZERO {
                let weight = -buy.to_f64().unwrap().ln();
                edges.push((*from, *to, weight));
            }
        }
        if let Some(sell) = rate.sell {
            if sell > Decimal::ZERO {
                let weight = -(Decimal::ONE / sell).to_f64().unwrap().ln();
                edges.push((*to, *from, weight));
            }
        }
    }
    const EPSILON: f64 = 1e-8;
    for _ in 0..num_currencies - 1 {
        for &(u, v, weight) in &edges {
            if dist[u] + weight < dist[v] - EPSILON {
                dist[v] = dist[u] + weight;
            }
        }
    }
    for &(u, v, weight) in &edges {
        if dist[u] + weight < dist[v] - EPSILON {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::acba;
    use rust_decimal_macros::dec;

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
        for (from, to) in &get_conversations() {
            let _ = find_all_paths(&graph, from, to);
        }
        Ok(())
    }

    #[test]
    fn test_arbitrage_true_buy() {
        let rates = vec![
            Rate {
                from: Currency::usd(),
                to: Currency::eur(),
                rate_type: RateType::NoCash,
                buy: Some(dec!(0.83)),
                sell: Some(dec!(0.85)),
            },
            Rate {
                from: Currency::eur(),
                to: Currency::new("CHF"),
                rate_type: RateType::NoCash,
                buy: Some(dec!(0.88)),
                sell: Some(dec!(0.9)),
            },
            Rate {
                from: Currency::new("CHF"),
                to: Currency::usd(),
                rate_type: RateType::NoCash,
                buy: Some(dec!(1.37)),
                sell: Some(dec!(1.5)),
            },
        ];
        assert!(detect_arbitrage(&rates, RateType::NoCash));
    }

    #[test]
    fn test_arbitrage_true_sell() {
        let rates = vec![
            Rate {
                from: Currency::eur(),
                to: Currency::usd(),
                rate_type: RateType::NoCash,
                buy: Some(dec!(1.1)),
                sell: Some(dec!(1.2)),
            },
            Rate {
                from: Currency::usd(),
                to: Currency::new("CHF"),
                rate_type: RateType::NoCash,
                buy: Some(dec!(0.7)),
                sell: Some(dec!(0.75)),
            },
            Rate {
                from: Currency::new("CHF"),
                to: Currency::eur(),
                rate_type: RateType::NoCash,
                buy: Some(dec!(1.05)),
                sell: Some(dec!(1.1)),
            },
        ];
        assert!(detect_arbitrage(&rates, RateType::NoCash));
    }

    #[test]
    fn test_arbitrage_false() {
        let rates = vec![
            Rate {
                from: Currency::usd(),
                to: Currency::eur(),
                rate_type: RateType::NoCash,
                buy: Some(dec!(0.8)),
                sell: Some(dec!(0.85)),
            },
            Rate {
                from: Currency::eur(),
                to: Currency::new("CHF"),
                rate_type: RateType::NoCash,
                buy: Some(dec!(0.9)),
                sell: Some(dec!(0.95)),
            },
            Rate {
                from: Currency::new("CHF"),
                to: Currency::usd(),
                rate_type: RateType::NoCash,
                buy: Some(dec!(1.38)),
                sell: Some(dec!(1.4)),
            },
        ];
        assert!(!detect_arbitrage(&rates, RateType::NoCash));
    }
}
