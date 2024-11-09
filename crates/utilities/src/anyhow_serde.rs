use anyhow::Result;
use serde::{Deserialize, Serialize};

pub fn from_str<T: for<'a> Deserialize<'a>>(line: &String) -> Result<T> {
    return serde_json::from_str(line).map_err(|err| anyhow::Error::new(err));
}

pub fn to_string<T: Serialize>(value: &T) -> Result<String> {
    return serde_json::to_string(value).map_err(|err| anyhow::Error::new(err));
}
