use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ERC20TokenPriceResponse {
    token_name: String,
    token_symbol: String,
    token_logo: String,
    token_decimals: String,
    native_price: NativePrice,
    pub usd_price: f64,
    usd_price_formatted: String,
    exchange_name: String,
    exchange_address: String,
    token_address: String,
    price_last_changed_at_block: String,
    verified_contract: bool,
    #[serde(rename = "24hr_percent_change")]
    hr24_percent_change: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NativePrice {
    value: String,
    decimals: u8,
    name: String,
    symbol: String,
    address: String,
}
