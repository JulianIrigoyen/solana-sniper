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
    previousBlockhash: String,
    blockhash: String,
    parentSlot: u64,
    transactions: Vec<Transaction>,
    // Add more fields as necessary
    blockTime: Option<u64>,
    blockHeight: Option<u64>,
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
    preBalances: Vec<u64>,
    postBalances: Vec<u64>,
    innerInstructions: Vec<InnerInstruction>,
    // Add more fields as necessary
    logMessages: Option<Vec<String>>,
    preTokenBalances: Option<Vec<TokenBalance>>,
    postTokenBalances: Option<Vec<TokenBalance>>,
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
    programIdIndex: u8,
    accounts: Vec<u8>,
    data: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct TokenBalance {
    accountIndex: u8,
    mint: String,
    uiTokenAmount: UiTokenAmount,
    owner: String,
    programId: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct UiTokenAmount {
    uiAmount: Option<f64>,
    decimals: u8,
    amount: String,
    uiAmountString: String,
}
