use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SolanaTransaction {
    pub jsonrpc: String,
    pub result: TransactionResult,
    pub id: u64,
    pub block_time: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionResult {
    pub meta: TransactionMeta,
    pub slot: u64,
    pub transaction: TransactionDetail,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionMeta {
    pub err: Option<serde_json::Value>,
    pub fee: u64,
    #[serde(default)]
    pub inner_instructions: Vec<serde_json::Value>,
    pub post_balances: Vec<u64>,
    #[serde(default)]
    pub post_token_balances: Vec<serde_json::Value>,
    pub pre_balances: Vec<u64>,
    #[serde(default)]
    pub pre_token_balances: Vec<serde_json::Value>,
    #[serde(default)]
    pub rewards: Vec<serde_json::Value>,
    pub status: HashMap<String, Option<serde_json::Value>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionDetail {
    pub message: TransactionMessage,
    pub signatures: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionMessage {
    pub account_keys: Vec<String>,
    pub header: TransactionHeader,
    pub instructions: Vec<TransactionInstruction>,
    pub recent_blockhash: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionHeader {
    pub num_readonly_signed_accounts: u8,
    pub num_readonly_unsigned_accounts: u8,
    pub num_required_signatures: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionInstruction {
    pub accounts: Vec<u8>,
    pub data: String,
    pub program_id_index: u8,
}