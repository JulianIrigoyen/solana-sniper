use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::str::FromStr;
use actix_web::{web, HttpResponse, Responder};
use diesel::serialize::IsNull::No;
use log::Level::Debug;
use reqwest::{Client, header};
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use solana_sdk::bs58;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use rust_decimal::prelude::{One, Zero};
use crate::server::endpoints::holders;

use crate::server::endpoints::whales::get_token_supply;

pub fn init_routes(cfg: &mut web::ServiceConfig){
    cfg.service(web::resource("/transactions")
        .route(web::post().to(find_transactions))
    );
}

#[derive(Serialize, Deserialize, Debug)]
struct FindTransactionsRequest {
    transaction_signatures: Vec<String>
}

async fn find_transactions(request: web::Json<FindTransactionsRequest>) -> impl Responder {
    let transaction_data = process_transaction_signatures(request.transaction_signatures.clone()).await;
    match transaction_data {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

/**
Returns transaction details for a confirmed transaction. Params:
 - Transaction signature vector, as base-58 encoded strings
 */
async fn process_transaction_signatures(signatures: Vec<String>) -> Result<Vec<TransactionResponse>, Box<dyn Error>> {

    let mut request_headers = header::HeaderMap::new();
    request_headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
    let rpc_url = "https://api.mainnet-beta.solana.com";//env::var("PRIVATE_SOLANA_QUICKNODE").expect("PRIVATE_SOLANA_QUICKNODE must be set");

    println!("Finding transactions for signatures {:#?}", signatures);
    let mut transactions: Vec<TransactionResponse>= Vec::new();
    let client = Client::new();

    for signature in signatures {
        let params = json!([
            signature,
            {
                "encoding": "jsonParsed"
            }
        ]);

        let rpc_request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getTransaction",
            "params": params
        });

       
        let response = client
            .post(rpc_url.clone())
            .headers(request_headers.clone())
            .json(&rpc_request)
            .send()
            .await?;

        if response.status().is_success() {
            let response_text = response.text().await?;
            // TODO most useful print ever
            let value: serde_json::Value = serde_json::from_str(&response_text)?;
            println!("{:#?}", value);

            //TODO FINISH THIS
        }
    }

    Ok(transactions)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct TransactionResponse {
    jsonrpc: String,
    result: TransactionResult,
    block_time: Option<u64>,
    id: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct TransactionResult {
    meta: TransactionMeta,
    slot: u64,
    transaction: TransactionDetails,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct TransactionMeta {
    err: Option<serde_json::Value>,
    fee: u64,
    inner_instructions: Vec<serde_json::Value>,
    post_balances: Vec<u64>,
    post_token_balances: Vec<serde_json::Value>,
    pre_balances: Vec<u64>,
    pre_token_balances: Vec<serde_json::Value>,
    rewards: Vec<serde_json::Value>,
    status: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct TransactionDetails {
    message: TransactionMessage,
    signatures: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct TransactionMessage {
    account_keys: Vec<String>,
    header: TransactionHeader,
    instructions: Vec<TransactionInstruction>,
    recent_blockhash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct TransactionHeader {
    num_readonly_signed_accounts: u8,
    num_readonly_unsigned_accounts: u8,
    num_required_signatures: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct TransactionInstruction {
    accounts: Vec<u8>,
    data: String,
    program_id_index: u8,
}
