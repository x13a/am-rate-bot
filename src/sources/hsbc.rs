use crate::sources::{Currency, Error, Rate, RateType};
use select::document::Document;
use select::predicate::{Attr, Name};

pub const API_URL: &str = "https://www.hsbc.am/en-am/help/rates/";

#[derive(Debug)]
pub struct Response {
    pub rates: Vec<Rate>,
}

impl Response {
    pub fn url() -> String {
        API_URL.into()
    }

    pub async fn get_rates(c: &reqwest::Client) -> Result<Self, Error> {
        let html = c.get(Self::url()).send().await?.text().await?;
        let mut rates = vec![];
        let document = Document::from(html.as_str());
        let table = document
            .find(Attr("id", "content_main_basicTable_1"))
            .next()
            .ok_or(Error::Html)?
            .find(Name("table"))
            .filter(|v| v.attr("class").is_some_and(|s| s == "desktop"))
            .next()
            .ok_or(Error::Html)?;
        for row in table
            .find(Name("tbody"))
            .next()
            .ok_or(Error::Html)?
            .find(Name("tr"))
        {
            let mut cells = row.find(Name("td"));
            let currency = cells.next().ok_or(Error::Html)?.text();
            let buy = cells.next().ok_or(Error::Html)?.text();
            let sell = cells.next().ok_or(Error::Html)?.text();
            let buy_cash = cells.next().ok_or(Error::Html)?.text();
            let sell_cash = cells.next().ok_or(Error::Html)?.text();
            rates.push(Rate {
                from: Currency::from(&currency),
                to: Currency::default(),
                rate_type: RateType::NoCash,
                buy: buy.trim().parse().ok(),
                sell: sell.trim().parse().ok(),
            });
            rates.push(Rate {
                from: Currency::from(&currency),
                to: Currency::default(),
                rate_type: RateType::Cash,
                buy: buy_cash.trim().parse().ok(),
                sell: sell_cash.trim().parse().ok(),
            });
        }
        let result = Self { rates };
        Ok(result)
    }
}
