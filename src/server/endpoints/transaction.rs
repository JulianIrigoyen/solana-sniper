use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct TransactionResponse {
    jsonrpc: String,
    result: TransactionResult,
    id: u64,
    block_time: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TransactionResult {
    meta: TransactionMeta,
    transaction: TransactionDetail,
    slot: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct TransactionMeta {
    err: Option<serde_json::Value>,
    fee: u64,
    inner_instructions: Vec<serde_json::Value>, // Adjust type if you have a detailed format
    post_balances: Vec<u64>,
    post_token_balances: Vec<serde_json::Value>, // Adjust type if you have a detailed format
    pre_balances: Vec<u64>,
    pre_token_balances: Vec<serde_json::Value>, // Adjust type if you have a detailed format
    rewards: Vec<serde_json::Value>, // Adjust type if you have a detailed format
    status: TransactionStatus,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum TransactionStatus {
    Ok(Option<serde_json::Value>),
    Err(Option<serde_json::Value>), // Replace with specific error type if available
}

#[derive(Serialize, Deserialize, Debug)]
struct TransactionDetail {
    message: TransactionMessage,
    signatures: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TransactionMessage {
    account_keys: Vec<String>,
    header: TransactionHeader,
    instructions: Vec<Instruction>,
    recent_blockhash: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct TransactionHeader {
    num_readonly_signed_accounts: u8,
    num_readonly_unsigned_accounts: u8,
    num_required_signatures: u8,
}

#[derive(Serialize, Deserialize, Debug)]
struct Instruction {
    accounts: Vec<u8>,
    data: String,
    program_id_index: u8,
}