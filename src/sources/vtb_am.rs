use crate::sources::{Currency, Error, Rate, RateType, RatesConfigTrait};
pub use crate::sources::{RatesConfig as Config, Response};
use select::{
    document::Document,
    predicate::{Class, Name},
};

pub async fn get<T>(client: &reqwest::Client, config: &T) -> anyhow::Result<Response>
where
    T: RatesConfigTrait,
{
    let html = client.get(config.rates_url()).send().await?.text().await?;
    let document = Document::from(html.as_str());
    let mut rates = vec![];
    for (idx, exchange_table) in document.find(Class("exchange-rate-table")).enumerate() {
        for row in exchange_table
            .find(Name("table"))
            .next()
            .ok_or(Error::Html)?
            .find(Name("tbody"))
            .next()
            .ok_or(Error::Html)?
            .find(Name("tr"))
        {
            let mut cells = row.find(Name("td"));
            let span = Name("span");
            let currency = cells
                .next()
                .ok_or(Error::Html)?
                .find(span)
                .next()
                .ok_or(Error::Html)?
                .text();
            let buy = cells
                .next()
                .ok_or(Error::Html)?
                .find(span)
                .next()
                .ok_or(Error::Html)?
                .text();
            let sell = cells
                .next()
                .ok_or(Error::Html)?
                .find(span)
                .next()
                .ok_or(Error::Html)?
                .text();
            rates.push(Rate {
                from: Currency::new(currency),
                to: Currency::default(),
                rate_type: match idx {
                    0 => RateType::Cash,
                    _ => RateType::NoCash,
                },
                buy: buy.trim().parse().ok(),
                sell: sell.trim().parse().ok(),
            });
        }
    }
    Ok(Response { rates })
}
