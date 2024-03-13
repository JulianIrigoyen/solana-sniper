use std::collections::HashMap;
use std::error::Error;
use reqwest::Client;
use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionSummary {
    pub program_id: String,
    pub amount_delta: f64,
    pub mint: String,
    pub owner: String,
    // Token name and symbol are placeholders for now
    pub token_name: String,
    pub token_symbol: String,
}

impl SolanaTransaction {
    pub fn to_summary(&self) -> Vec<TransactionSummary> {
        let mut summaries = Vec::new();

        if let Some(result) = self.result.as_ref() {
            let pre_balances_map = result.meta.pre_token_balances.iter()
                .map(|balance| {
                    let mint = &balance.mint;
                    let owner = &balance.owner;
                    ((mint, owner), balance)
                })
                .collect::<HashMap<_, _>>();

            for post_balance in &result.meta.post_token_balances {
                let post_mint = &post_balance.mint;
                let post_owner = &post_balance.owner;
                let post_amount: f64 = post_balance.ui_token_amount.amount.parse().unwrap_or(0.0);

                if let Some(pre_balance) = pre_balances_map.get(&(post_mint, post_owner)) {
                    let pre_amount: f64 = pre_balance.ui_token_amount.amount.parse().unwrap_or(0.0);
                    let amount_delta = post_amount - pre_amount;

                    if amount_delta.abs() > std::f64::EPSILON {
                        summaries.push(TransactionSummary {
                            program_id: post_balance.program_id.clone(),
                            amount_delta,
                            mint: post_mint.clone(),
                            owner: post_owner.clone(),
                            token_name: "Unknown".to_string(), // Placeholder
                            token_symbol: "???".to_string(), // Placeholder
                        });
                    }
                }
            }
        }

        summaries
    }
}

async fn fetch_mint_metadata(client: &Client, mint_address: &str) -> Result<(String, String), Box<dyn Error>> {
    let response = client
        .get(&format!("https://api.example.com/mint/{}", mint_address))
        .send()
        .await?;

    if response.status().is_success() {
        let data = response.json::<serde_json::Value>().await?;
        let token_name = data["name"].as_str().unwrap_or_default().to_string();
        let token_symbol = data["symbol"].as_str().unwrap_or_default().to_string();
        Ok((token_name, token_symbol))
    } else {
        Err(Box::try_from("err").unwrap())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SolanaTransaction {
    pub jsonrpc: String,
    pub result: Option<TransactionResult>,

    pub id: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionResult {
    pub meta: TransactionMeta,
    pub slot: u64,
    pub transaction: TransactionDetail,
    pub block_time: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionMeta {
    pub err: Option<serde_json::Value>,
    pub fee: u64,
    #[serde(default)]
    pub inner_instructions: Vec<serde_json::Value>,
    pub post_balances: Option<Vec<u64>>,
    #[serde(default)]
    pub post_token_balances: Vec<TokenBalance>,
    pub pre_balances: Option<Vec<u64>>,
    #[serde(default)]
    pub pre_token_balances: Vec<TokenBalance>,
    #[serde(default)]
    pub rewards: Vec<serde_json::Value>,
    pub status: HashMap<String, Option<serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenBalance {
    pub account_index: u64,
    pub mint: String,
    pub owner: String,
    pub program_id: String,
    #[serde(rename = "uiTokenAmount")]
    pub ui_token_amount: UITokenAmount,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UITokenAmount {
    pub amount: String,
    pub decimals: u8,
    #[serde(rename = "uiAmount")]
    pub ui_amount: Option<f64>,
    #[serde(rename = "uiAmountString")]
    pub ui_amount_string: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionDetail {
    pub message: TransactionMessage,
    pub signatures: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionMessage {
    pub account_keys: Option<Vec<String>>,
    pub header: Option<TransactionHeader>,
    #[serde(default)]
    pub instructions: Vec<TransactionInstruction>,
    pub recent_blockhash: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionHeader {
    pub num_readonly_signed_accounts: u8,
    pub num_readonly_unsigned_accounts: u8,
    pub num_required_signatures: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionInstruction {
    #[serde(default)]
    pub accounts: Vec<String>,
    pub data: Option<String>,
    pub program_id_index: Option<u8>,
}