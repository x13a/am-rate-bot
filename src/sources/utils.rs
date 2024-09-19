use crate::sources::Currency;
use serde::{Deserialize, Deserializer};
use std::str::FromStr;

pub(crate) fn de_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let mut s = String::deserialize(deserializer)?;
    for c in &[",", "%"] {
        s = s.replace(c, "");
    }
    let f = s.parse::<f64>().map_err(serde::de::Error::custom)?;
    Ok(f)
}

pub(crate) fn de_currency<'de, D>(deserializer: D) -> Result<Currency, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(Currency::from_str(&s).expect("currency not supported"))
}
