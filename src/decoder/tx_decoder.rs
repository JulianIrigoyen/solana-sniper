//
// use
// anchor_client::{anchor_lang::prelude::*, Client as AnchorClient};
// use solana_sdk::{signature::Keypair, signer::Signer};
//
// use std::collections::HashMap;
// use std::env;
// use std::error::Error;
// use std::str::FromStr;
// use actix_web::{web, HttpResponse, Responder};
// use diesel::serialize::IsNull::No;
// use eyre::anyhow;
// use log::Level::Debug;
// use reqwest::{Client as RpcClient, Client, header};
// use serde::{Serialize, Deserialize};
// use serde_json::{json, Value};
// use solana_sdk::bs58;
// use rust_decimal::{prelude::FromPrimitive, Decimal};
// use rust_decimal::prelude::{One, Zero};
//
// // https://solana.stackexchange.com/questions/2635/fetch-onchain-idl-from-program-id
//
// // Decodes transaction instruction data for transactions on the Solana Blockchain
//
// use solana_sdk::commitment_config::CommitmentConfig;
//
// async fn get_program_idl(program_id_str: &str) -> Result<Value> {
//     let program_pubkey = Pubkey::from_str(program_id_str)?;
//
//     // Set up the Solana RPC client. You need to replace `http://localhost:8899` with the RPC endpoint you're using.
//     let rpc_url = "https://api.mainnet-beta.solana.com";//env::var("PRIVATE_SOLANA_QUICKNODE").expect("PRIVATE_SOLANA_QUICKNODE must be set");
//
//     println!("Fetching IDL for program {:#?}", signatures);
//     let mut transactions: Vec<TransactionResponse>= Vec::new();
//     let client = Client::new();
//
//     // Calculate the PDA for the IDL account.
//     let seeds = &[b"anchor:idl".as_ref(), program_pubkey.as_ref()];
//     let (idl_address, _) = Pubkey::find_program_address(seeds, &program_pubkey);
//
//     // Fetch the account data from the blockchain.
//     let account_data = rpc_client.get_account_data(&idl_address)
//         .map_err(|e| anyhow!("Failed to get account data: {}", e))?;
//
//     // The IDL data is compressed using zstd. First, skip the 8-byte discriminator.
//     let idl_data = &account_data[8..];
//
//     // Decompress the IDL data.
//     let decompressed_idl = decode_all(idl_data)
//         .map_err(|e| anyhow!("Failed to decompress IDL data: {}", e))?;
//
//     // Deserialize the JSON IDL.
//     let idl_json: Value = serde_json::from_slice(&decompressed_idl)
//         .map_err(|e| anyhow!("Failed to deserialize IDL: {}", e))?;
//
//     Ok(idl_json)
// }