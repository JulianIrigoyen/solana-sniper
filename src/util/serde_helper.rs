use serde::de::DeserializeOwned;
use serde_json::Value;
use std::error::Error;

pub fn deserialize_into<T: DeserializeOwned>(value: &Value) -> Result<T, Box<dyn Error>> {

    serde_json::from_value(value.clone()).map_err(|e| e.into())
}