use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use reqwest::Client;
use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionSummary {
    pub signature: String,
    pub program_id: String,
    pub amount_delta: f64,
    pub mint: String,
    pub owner: String,
    // Token name and symbol are placeholders for now
    pub token_name: String,
    pub token_symbol: String,
    pub transaction_type: String,
}

impl fmt::Display for TransactionSummary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Transaction {} Summary:\n\
                Program ID: {}\n\
                Amount Delta: {}\n\
                Mint: {}\n\
                Owner: {}\n\
                Token Name: {}\n\
                Token Symbol: {}\n\
                Transaction Type: {}",
               self.signature,
               self.program_id,
               self.amount_delta,
               self.mint,
               self.owner,
               self.token_name,
               self.token_symbol,
               self.transaction_type
        )
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TxCheckedSummary {
    pub signature: String,
    pub transaction_type: String,
    pub source: String, // the public key of the account from which the tokens are being transferred
    pub destination: String, // the public key of the account that will receive the tokens
    pub mint: String, // The public key  of the token's mint. This identifies which token is being transferred.
    pub token_name: String,
    pub token_symbol: String,
    pub token_amount: Option<f64>, // use uiAmount ->  field that presents the amount in a format that's ready for display, taking into account the decimals. It represents the amount of tokens being transferred as a floating-point number.
    pub token_price: Option<f64>,
    pub transaction_usd_value: Option<f64>,
    pub price_change_24_h: Option<f64>,
    pub liquidity: Option<f64>,
    pub detail: String, // use uiAmount ->  field that presents the amount in a format that's ready for display, taking into account the decimals. It represents the amount of tokens being transferred as a floating-point number.
}


impl fmt::Display for TxCheckedSummary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let solscan_link = format!("https://solscan.io/tx/{}", self.signature);
        let token_amount = if self.token_amount.is_some() { self.token_amount.unwrap() } else { 0.0 };
        let token_price = if self.token_price.is_some() { self.token_price.unwrap() } else { 0.0 };
        let price_change_24_h = if self.price_change_24_h.is_some() { self.price_change_24_h.unwrap() } else { 0.0 };
        let liquidity = if self.liquidity.is_some() { self.liquidity.unwrap() } else { 0.0 };
        let transaction_value = token_price * token_amount;
        write!(f, "Transaction Checked - {} - Summary:\n\
                Transaction Type: {}\n\
                Source: {}\n\
                Destination: {}\n\
                Mint: {}\n\
                Token Name: {}\n\
                Token Symbol: {}\n\
                Token Amount: {}\n\
                Token Price: {}\n\
                Transaction Value (USD): {}\n\
                Price Change (24h): {}\n\
                Liquidity: {}\n\
                Detail: {}\n\
                Link: {}",
               self.signature,
               self.transaction_type,
               self.source,
               self.destination,
               self.mint,
               self.token_name,
               self.token_symbol,
               token_amount,
               token_price,
               transaction_value,
               price_change_24_h,
               liquidity,
               self.detail,
               solscan_link
        )
    }
}
