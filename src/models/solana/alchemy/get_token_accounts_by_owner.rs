use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct TokenAccountsByOwnerResponse {
    jsonrpc: String,
    result: TokenAccountsByOwnerResult,
    id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct TokenAccountsByOwnerResult {
    context: Context,
    value: Vec<TokenAccount>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Context {
    slot: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct TokenAccount {
    pubkey: String,
    account: AccountDetail,
}

#[derive(Serialize, Deserialize, Debug)]
struct AccountDetail {
    data: AccountData,
    executable: bool,
    lamports: u64,
    owner: String,
    rentEpoch: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct AccountData {
    program: String,
    parsed: ParsedData,
    space: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct ParsedData {
    accountType: String,
    info: TokenAccountInfo,
    #[serde(rename = "type")]
    data_type: String, // Use `data_type` because `type` is a reserved keyword in Rust.
}

#[derive(Serialize, Deserialize, Debug)]
struct TokenAccountInfo {
    tokenAmount: TokenAmount,
    delegate: Option<String>, // This can be optional based on the account state.
    delegatedAmount: Option<TokenAmount>, // This can also be optional.
    state: String,
    isNative: bool,
    mint: String,
    owner: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct TokenAmount {
    amount: String,
    decimals: u8,
    uiAmount: f64,
    uiAmountString: String,
}
