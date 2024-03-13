use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
struct BlockNotificationResponse {
    jsonrpc: String,
    method: String,
    params: BlockNotificationParams,
}

#[derive(Serialize, Deserialize, Debug)]
struct BlockNotificationParams {
    result: BlockNotificationResult,
    subscription: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct BlockNotificationResult {
    context: Context,
    value: BlockValue,
}

#[derive(Serialize, Deserialize, Debug)]
struct Context {
    slot: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct BlockValue {
    slot: u64,
    block: Block,
}

#[derive(Serialize, Deserialize, Debug)]
struct Block {
    previous_blockhash: String,
    blockhash: String,
    parent_slot: u64,
    transactions: Vec<Transaction>,
    block_time: Option<u64>,
    block_height: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Transaction {
    transaction: Vec<String>, // This could be further detailed based on the actual structure
    meta: TransactionMeta,
}

#[derive(Serialize, Deserialize, Debug)]
struct TransactionMeta {
    err: Option<Value>,
    status: TransactionStatus,
    fee: u64,
    pre_balances: Vec<u64>,
    post_balances: Vec<u64>,
    inner_instructions: Vec<InnerInstruction>,
    // Add more fields as necessary
    log_messages: Option<Vec<String>>,
    pre_token_balances: Option<Vec<TokenBalance>>,
    post_token_balances: Option<Vec<TokenBalance>>,
    rewards: Option<Vec<Value>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TransactionStatus {
    Ok: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct InnerInstruction {
    index: u8,
    instructions: Vec<InstructionDetail>,
}

#[derive(Serialize, Deserialize, Debug)]
struct InstructionDetail {
    program_id_index: u8,
    accounts: Vec<u8>,
    data: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct TokenBalance {
    account_index: u8,
    mint: String,
    ui_token_amount: UiTokenAmount,
    owner: String,
    program_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct UiTokenAmount {
    ui_amount: Option<f64>,
    decimals: u8,
    amount: String,
    ui_amount_string: String,
}
