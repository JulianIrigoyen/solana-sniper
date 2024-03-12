// https://solana.com/docs/rpc/websocket/accountsubscribe
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SolanaAccountNotification {
    jsonrpc: String,
    method: String,
    params: Params,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Params {
    result: Result,
    subscription: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Result {
    context: Context,
    value: ResultValue,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Context {
    slot: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResultValue {
    data: Data,
    executable: bool,
    lamports: u64,
    owner: String,
    rent_epoch: u64,
    space: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Data {
    program: String,
    parsed: Parsed,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Parsed {
    #[serde(rename = "type")]
    type_field: String,
    info: Info,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Info {
    authority: String,
    blockhash: String,
    fee_calculator: FeeCalculator,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FeeCalculator {
    lamports_per_signature: u64,
}

impl Default for SolanaAccountNotification {
    fn default() -> Self {
        Self {
            jsonrpc: "".to_string(),
            method: "".to_string(),
            params: Params::default(),
        }
    }
}

impl Default for Params {
    fn default() -> Self {
        Self {
            result: Result::default(),
            subscription: 0,
        }
    }
}

impl Default for Result {
    fn default() -> Self {
        Self {
            context: Context::default(),
            value: ResultValue::default(),
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self {
            slot: 0,
        }
    }
}

impl Default for ResultValue {
    fn default() -> Self {
        Self {
            data: Data::default(),
            executable: false,
            lamports: 0,
            owner: "".to_string(),
            rent_epoch: 0,
            space: 0,
        }
    }
}

impl Default for Data {
    fn default() -> Self {
        Self {
            program: "".to_string(),
            parsed: Parsed::default(),
        }
    }
}

impl Default for Parsed {
    fn default() -> Self {
        Self {
            type_field: "".to_string(),
            info: Info::default(),
        }
    }
}

impl Default for Info {
    fn default() -> Self {
        Self {
            authority: "".to_string(),
            blockhash: "".to_string(),
            fee_calculator: FeeCalculator::default(),
        }
    }
}

impl Default for FeeCalculator {
    fn default() -> Self {
        Self {
            lamports_per_signature: 0,
        }
    }
}

use std::fmt;

impl fmt::Display for SolanaAccountNotification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Simplified example: Customize according to what you want to display
        write!(f, "Method: {}, Subscription: {}", self.method, self.params.subscription)
    }
}