use reqwest;

use serde::de::{self, Deserialize, Deserializer};
use serde_xml_rs::deserialize;
use std::fmt::{Display, Debug};
use std::str::FromStr;
use std::result::Result as StdResult;

use errors::*;

pub trait Contents {
    fn is_empty(&self) -> bool;
}

// Explicit deserialization converter from a String to a FromStr-implementer
// https://github.com/serde-rs/json/issues/317
pub fn from_string<'de, T, D>(deserializer: D) -> StdResult<T, D::Error>
    where T: FromStr,
          T::Err: Display,
          D: Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
}

pub fn download<'de, T>(url: &String) -> Result<Option<T>> where
    T: Deserialize<'de> + Debug + Contents {

    let mut response = reqwest::get(&url[..])?;
    let body = response.text()?;
    let date = response.headers().get(reqwest::header::DATE).unwrap().to_str().unwrap();
    let status = response.status();
    match status {
        reqwest::StatusCode::OK => {
            debug!(r#"request="{}" response="{}" response_date="{}""#, url, status, date);
            deserialize(body.as_bytes())
                .and_then(|d: T| {
                    if d.is_empty() {
                        Ok(None)
                    }
                    else {
                        Ok(Some(d))
                    }
                })
                .chain_err(|| "Deserialization failed.")
        },
        _ => {
            warn!(r#"request="{}" response="{}" response_date="{}""#, url, status, date);
            return Err(format!("Bad response: {}", status).into());
        },
    }

}
