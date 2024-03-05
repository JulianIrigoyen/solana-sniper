use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ProgramAccountsResponse {
    jsonrpc: String,
    result: Vec<Account>,
    id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Account {
    pubkey: String,
    account: AccountData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountData {
    lamports: u64,
    owner: String,
    data: Vec<String>, // or another structure depending on the encoding
    executable: bool,
    rent_epoch: u64,
}