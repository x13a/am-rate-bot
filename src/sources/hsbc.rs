pub use crate::sources::SourceConfig as Config;
use crate::sources::{Currency, Error, Rate, RateType, SourceConfigTrait};
use select::document::Document;
use select::predicate::{Attr, Name};

#[derive(Debug)]
pub struct Response {
    pub rates: Vec<Rate>,
}

impl Response {
    pub async fn get_rates<T>(client: &reqwest::Client, config: &T) -> anyhow::Result<Self>
    where
        T: SourceConfigTrait,
    {
        let html = client.get(config.rates_url()).send().await?.text().await?;
        let document = Document::from(html.as_str());
        let table = document
            .find(Attr("id", "content_main_basicTable_1"))
            .next()
            .ok_or(Error::Html)?
            .find(Name("table"))
            .filter(|v| v.attr("class").is_some_and(|s| s == "desktop"))
            .next()
            .ok_or(Error::Html)?;
        let mut rates = vec![];
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
            let to = Currency::default();
            let from = Currency::new(currency);
            rates.push(Rate {
                from: from.clone(),
                to: to.clone(),
                rate_type: RateType::NoCash,
                buy: buy.trim().parse().ok(),
                sell: sell.trim().parse().ok(),
            });
            rates.push(Rate {
                from,
                to,
                rate_type: RateType::Cash,
                buy: buy_cash.trim().parse().ok(),
                sell: sell_cash.trim().parse().ok(),
            });
        }
        Ok(Self { rates })
    }
}
