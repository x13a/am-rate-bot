use crate::sources::Currency;
use serde::{de, Deserialize, Deserializer};
use serde_json::Value;
use std::str::FromStr;

pub(crate) fn de_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let mut s = String::deserialize(deserializer)?;
    for c in [",", "%"] {
        s = s.replace(c, "");
    }
    let f = s.parse::<f64>().map_err(de::Error::custom)?;
    Ok(f)
}

pub(crate) fn de_currency<'de, D>(deserializer: D) -> Result<Currency, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(Currency::from_str(&s).expect("currency not supported"))
}

pub(crate) fn de_u8<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    let rv = match Value::deserialize(deserializer)? {
        Value::String(s) => s.parse().map_err(de::Error::custom)?,
        Value::Number(n) => n.as_u64().ok_or(de::Error::custom(""))? as u8,
        _ => Err(de::Error::custom(""))?,
    };
    Ok(rv)
}
