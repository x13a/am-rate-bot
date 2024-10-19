use crate::sources::{Currency, Error, Rate, RateType, RatesConfigTrait};
pub use crate::sources::{RatesConfig as Config, Response};
use select::{document::Document, predicate::Class};

pub async fn get<T>(client: &reqwest::Client, config: &T) -> anyhow::Result<Response>
where
    T: RatesConfigTrait,
{
    let html = client.get(config.rates_url()).send().await?.text().await?;
    let document = Document::from(html.as_str());
    let exchange_table = document
        .find(Class("exchange-table"))
        .next()
        .ok_or(Error::Html)?;
    let mut rates = vec![];
    for row in exchange_table.find(Class("exchange-table__row")).skip(1) {
        let mut cells = row.find(Class("exchange-table__cell-content"));
        let currency = cells.next().ok_or(Error::Html)?.text();
        let buy = cells.next().ok_or(Error::Html)?.text();
        let sell = cells.next().ok_or(Error::Html)?.text();
        let rate = Rate {
            from: Currency::new(currency),
            to: Currency::default(),
            rate_type: RateType::Cash,
            buy: buy.trim().parse().ok(),
            sell: sell.trim().parse().ok(),
        };
        rates.push(rate);
    }
    Ok(Response { rates })
}
