pub use crate::source::{BaseConfig as Config, BaseResponse as Response};
use crate::source::{BaseConfigTrait, Currency, Error, Rate, RateType};
use select::{
    document::Document,
    predicate::{Attr, Name},
};

pub async fn get<T>(client: &reqwest::Client, config: &T) -> anyhow::Result<Response>
where
    T: BaseConfigTrait,
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
    Ok(Response { rates })
}

pub async fn collect<T>(client: &reqwest::Client, config: &T) -> anyhow::Result<Vec<Rate>>
where
    T: BaseConfigTrait,
{
    let resp: Response = get(client, config).await?;
    Ok(resp.rates)
}
