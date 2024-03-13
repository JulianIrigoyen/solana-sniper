// use actix_web::web::block;
// use actix_web::{web, HttpResponse, Responder};
// use reqwest::{Client, header};
// use serde::{Serialize, Deserialize};
// use serde_json::{json, Value};
// use solana_sdk::bs58;
// use rust_decimal::{prelude::FromPrimitive, Decimal};
//
// use lazy_static::lazy_static;
// use hashbrown::HashMap;
// use std::os::macos::fs;
// use std::sync::Mutex;
//
// use std::{env, thread};
//
// use chrono::Local;
// use csv::Writer;
// use std::fs::File;
// use std::fs::create_dir_all;
// use std::path::{Path, PathBuf};
// use std::error::Error;
// use std::str::FromStr;
// use solana_client::rpc_client;
//
// use solana_client::rpc_client::RpcClient;
// use solana_client::rpc_config::RpcTransactionConfig;
// use solana_sdk::signature::{Signature, Signer};
// use solana_sdk::transaction::Transaction;
// use solana_sdk::commitment_config::CommitmentConfig;
// use solana_sdk::program_option::COption;
// use solana_sdk::pubkey::Pubkey;
//
//
// /**
// Detect New SPL Tokens: Incorporate a monitoring mechanism to scan the network for new token mint transactions.
// This could involve tracking the createAccount and initializeMint instructions within the Token Program.
//
// Resources:
// https://solana.stackexchange.com/questions/9228/how-to-find-a-new-token-in-the-block
// https://solana.com/es/docs/rpc/http/getsignaturesforaddress
// https://github.com/solana-labs/solana-program-library/blob/master/token/program/src/instruction.rs#L39-L45
//
//  */
//
// pub enum TokenInstruction {
//     InitializeMint {
//         decimals: u8,
//         mint_authority: Pubkey,
//         freeze_authority: COption<Pubkey>,
//     },
// }
//
// #[derive(Serialize, Deserialize, Debug)]
// struct InitializeMintData {
//     decimals: u8,
//     mint_authority: String, // Using String to represent Pubkey for simplicity in JSON
//     freeze_authority: Option<String>, // Optional String
// }
//
// #[derive(Serialize, Deserialize, Debug)]
// struct GetSignaturesResponse {
//     jsonrpc: String,
//     result: Vec<SignatureResult>,
//     id: u64,
// }
//
// #[derive(Serialize, Deserialize, Debug)]
// struct SignatureResult {
//     err: Option<Value>, // Use more specific type if error structure is known
//     memo: Option<String>,
//     signature: String,
//     slot: u64,
//     blockTime: Option<i64>,
// }
//
// pub fn init_routes(cfg: &mut web::ServiceConfig) {
//     cfg.service(
//         web::resource("/new-spls")
//             .route(web::get().to(find_new_spls))
//     );
// }
//
// #[derive(Serialize, Deserialize, Debug)]
// struct NewSplToken {
//     mint_address: String,
//     // Additional fields can be added as necessary
// }
//
// async fn find_new_spls() -> impl Responder {
//     let client = Client::new();
//     let solana_rpc_url = env::var("SOLANA_RPC_URL").expect("SOLANA_RPC_URL must be set");
//     let token_program_id = env::var("SOLANA_TOKEN_PROGRAM").expect("SOLANA_TOKEN_PROGRAM must be set");
//
//     // Fetch the recent transaction signatures for the given address
//     let resp = client.post(&solana_rpc_url)
//         .json(&serde_json::json!({
//             "jsonrpc": "2.0",
//             "id": 1,
//             "method": "getSignaturesForAddress",
//             "params": [
//                 token_program_id,
//                 {
//                     "limit": 10 // Adjust limit as needed
//                 }
//             ]
//         }))
//         .send()
//         .await?;
//
//     let signatures: Vec<String> = resp.json().await?; // Parse the response to extract transaction signatures
//     //
//     // let mut new_spls = Vec::new();
//     //
//     // for signature in signatures {
//     //     // Fetch transaction details
//     //     let transaction_resp = client.post(&solana_rpc_url)
//     //         .json(&serde_json::json!({
//     //             "jsonrpc": "2.0",
//     //             "id": 1,
//     //             "method": "getTransaction",
//     //             "params": [signature, "jsonParsed"]
//     //         }))
//     //         .send()
//     //         .await?;
//     //
//     //     let transaction: Value = transaction_resp.json().await?; // Parse the response into your Transaction struct
//     //
//     //     // Analyze the transaction to find InitializeMint instructions
//     //     for instruction in transaction.message.instructions {
//     //         if is_initialize_mint(&instruction) {
//     //             new_spls.push(NewSplToken {
//     //                 mint_address: instruction.program_id.clone(),
//     //                 // Populate other fields as needed
//     //             });
//     //         }
//     //     }
//     // }
//
//     Ok(())
// }
//
// fn is_initialize_mint(instruction: &Value) -> bool {
//     // Implement logic to check if the instruction is an InitializeMint instruction
//     // This involves checking the program_id and the instruction data format
//     true
// }
//
