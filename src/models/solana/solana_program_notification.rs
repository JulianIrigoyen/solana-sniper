use serde::{Deserialize, Serialize};
use serde_json::Value; // For handling flexible data structures

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SolanaProgramNotification {
    pub jsonrpc: String,
    pub method: String,
    pub params: ProgramNotificationParams,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProgramNotificationParams {
    pub result: ProgramNotificationResult,
    pub subscription: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProgramNotificationResult {
    pub context: ProgramNotificationContext,
    pub value: ProgramNotificationValue,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProgramNotificationContext {
    pub slot: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProgramNotificationValue {
    pub pubkey: String,
    pub account: ProgramAccount,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)] // Allows for different types of `data` field representations
pub enum ProgramAccountData {
    Base58(Vec<String>), // For base58 encoding: ["data", "base58"]
    ParsedJson {
        program: String,
        parsed: Value, // Flexible to accommodate any structure
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProgramAccount {
    pub data: ProgramAccountData,
    pub executable: bool,
    pub lamports: u64,
    pub owner: String,
    pub rentEpoch: u64,
    pub space: u64,
}

impl Default for SolanaProgramNotification {
    fn default() -> Self {
        SolanaProgramNotification {
            jsonrpc: "".to_string(),
            method: "".to_string(),
            params: ProgramNotificationParams::default(),
        }
    }
}

impl Default for ProgramNotificationParams {
    fn default() -> Self {
        ProgramNotificationParams {
            result: ProgramNotificationResult::default(),
            subscription: 0,
        }
    }
}

impl Default for ProgramNotificationResult {
    fn default() -> Self {
        ProgramNotificationResult {
            context: ProgramNotificationContext::default(),
            value: ProgramNotificationValue::default(),
        }
    }
}

impl Default for ProgramNotificationContext {
    fn default() -> Self {
        ProgramNotificationContext {
            slot: 0,
        }
    }
}

impl Default for ProgramNotificationValue {
    fn default() -> Self {
        ProgramNotificationValue {
            pubkey: "".to_string(),
            account: ProgramAccount::default(),
        }
    }
}

impl Default for ProgramAccount {
    fn default() -> Self {
        ProgramAccount {
            data: ProgramAccountData::Base58(vec![]), // Default to one of the possible types
            executable: false,
            lamports: 0,
            owner: "".to_string(),
            rentEpoch: 0,
            space: 0,
        }
    }
}
