pub use crate::sources::SourceConfig as Config;
use crate::sources::{Currency, Error, Rate, RateType, SourceConfigTrait};
use select::document::Document;
use select::predicate::Class;

#[derive(Debug)]
pub struct Response {
    pub rates: Vec<Rate>,
}

impl Response {
    pub async fn get<T>(client: &reqwest::Client, config: &T) -> anyhow::Result<Self>
    where
        T: SourceConfigTrait,
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
        Ok(Self { rates })
    }
}
