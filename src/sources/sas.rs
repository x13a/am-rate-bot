use crate::sources::{Currency, Error, Rate, RateType};
use select::document::Document;
use select::predicate::Class;

pub const API_URL: &str = "https://www.sas.am/app/";

#[derive(Debug)]
pub struct Response {
    pub rates: Vec<Rate>,
}

impl Response {
    pub fn url() -> String {
        API_URL.into()
    }

    pub async fn get_rates(c: &reqwest::Client) -> anyhow::Result<Self> {
        let html = c.get(Self::url()).send().await?.text().await?;
        let mut rates = vec![];
        let document = Document::from(html.as_str());
        let exchange_table = document
            .find(Class("exchange-table"))
            .next()
            .ok_or(Error::Html)?;
        for row in exchange_table.find(Class("exchange-table__row")).skip(1) {
            let mut cells = row.find(Class("exchange-table__cell-content"));
            let currency = cells.next().ok_or(Error::Html)?.text();
            let buy = cells.next().ok_or(Error::Html)?.text();
            let sell = cells.next().ok_or(Error::Html)?.text();
            let rate = Rate {
                from: Currency::from(currency),
                to: Currency::default(),
                rate_type: RateType::Cash,
                buy: buy.trim().parse().ok(),
                sell: sell.trim().parse().ok(),
            };
            rates.push(rate);
        }
        let result = Self { rates };
        Ok(result)
    }
}
