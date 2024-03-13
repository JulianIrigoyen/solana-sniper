// Required libraries:
// `crossbeam_channel` for efficient message passing between threads.
// `futures_util` for async stream processing.
// `timely` for dataflow-based stream processing.
// `tokio_tungstenite` for WebSocket communication.
// `url` for URL parsing.
// Custom model definitions for deserializing JSON messages.
#![allow(unused_variables)]
#[macro_use]
extern crate diesel;
extern crate serde;
extern crate serde_derive;
extern crate timely;

use std::{env, thread};
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::task::Context;

use crossbeam_channel::{bounded, Sender};
use dotenv::dotenv;
use futures_util::{sink::SinkExt, stream::StreamExt};
use reqwest::Client;
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use timely::dataflow::InputHandle;
use timely::dataflow::operators::{Filter, Input, Inspect};
use timely::execute_from_args;
use timely::worker::Worker;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async, MaybeTlsStream, tungstenite::protocol::Message, WebSocketStream,
};
use url::Url;
use crate::db::db_session_manager::DbSessionManager;

use crate::models::solana::solana_event_types::SolanaEventTypes;
use crate::models::solana::alchemy::get_program_accounts::ProgramAccountsResponse;
use crate::server::ws_server;
use crate::subscriber::websocket_subscriber::{WebSocketSubscriber, AuthMethod, SolanaSubscriptionBuilder};
// use crate::util::event_filters::{
//     EventFilters, FilterCriteria, FilterValue, ParameterizedFilter,
// };
use crate::subscriber::consume_stream::{consume_stream};
use crate::trackers::raydium::new_token_tracker;
use crate::trackers::raydium::new_token_tracker::NewTokenTracker;

use actix::prelude::*;
use crate::models::solana::solana_account_notification::SolanaAccountNotification;
use crate::models::solana::solana_transaction::{TransactionSummary, TxCheckedSummary};

mod db;
mod util;
mod models;
mod server;
mod http;
mod trackers;
mod schema;
mod subscriber;
mod decoder;

/**

Welcome to the Solana Sniper.

 */
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // let system = actix::System::new();


    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_session_manager = Arc::new(DbSessionManager::new(&database_url));

    // Websocket Server Initialization
    let ws_host = env::var("WS_SERVER_HOST").expect("WS_HOST must be set");
    let ws_port = env::var("WS_SERVER_PORT").expect("WS_PORT must be set");
    let ws_server_task = tokio::spawn(async move {
        let ws_server = ws_server::WebSocketServer::new(ws_host, ws_port);
        //TODO
        // ws_server.run().await
    });

    let solana_public_ws_url = String::from("wss://api.mainnet-beta.solana.com");
    let solana_private_ws_url =
        env::var("PRIVATE_SOLANA_QUICKNODE_WS").expect("PRIVATE_SOLANA_QUICKNODE_WS must be set");
    let solana_private_http_url = env::var("PRIVATE_SOLANA_QUICKNODE_HTTP")
        .expect("PRIVATE_SOLANA_QUICKNODE_HTTP must be set");

    // api key is provided in the path
    let (mut solana_ws_stream, _) = connect_async(solana_public_ws_url.clone()).await?;
    println!("Connected to Solana WebSocket");

    //https://solana.com/docs/rpc/websocket/accountsubscribe
    let solana_subscriber = WebSocketSubscriber::<SolanaSubscriptionBuilder>::new(
        solana_private_ws_url.to_string(),
        // solana_public_ws_url.to_string(),
        None,
        AuthMethod::None,
        SolanaSubscriptionBuilder,
    );

    // ------------ PROGRAM SUBSCRIPTION ------------
    let subscription_program_ids = vec![
        "EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm", // WIF
        "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263", // BONK
        "HJ39rRZ6ys22KdB3USxDgNsL7RKiQmsC3yL8AS3Suuku", // UPDOG
    ];

    let mut sub_program_params: Vec<(&str, Vec<String>)> = Vec::new();
    for id in subscription_program_ids {
        let param = (
            "programSubscribe",
            vec![
                id.to_string(),
                json!({
                    "encoding": "jsonParsed",
                    "commitment": "finalized"
                })
                    .to_string(),
            ],
        );
        sub_program_params.push(param);
    }

    // ------------ ACCOUNT SUBSCRIPTION ------------
    let account_program_ids: Vec<String> = vec![
        ("FJRZ5sTp27n6GhUVqgVkY4JGUJPjhRPnWtH4du5UhKbw".to_string()), //whale de miglio
        ("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1".to_string()), //raydium
        ("11111111111111111111111111111111".to_string()),             //system program
        ("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string()),  //token program
    ];

    let sub_accounts_message =
        json!({
              "jsonrpc": "2.0",
              "id": 1,
              "method": "accountSubscribe",
              "params": [
                "FJRZ5sTp27n6GhUVqgVkY4JGUJPjhRPnWtH4du5UhKbw",
                {
                  "encoding": "jsonParsed",
                  "commitment": "finalized"
                }
              ]
            }).to_string();


    println!("Subscribing to ACCOUNTS on  {} with provided messages :: {:?}", solana_private_ws_url.clone(), sub_accounts_message.clone());
    solana_ws_stream.send(Message::Text(sub_accounts_message)).await?;

    // ------------ LOG SUBSCRIPTION ------------
    let raydium_public_key = "srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX";
    let openbook_public_key = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";

    // --- WHALE KEYS ---
    let miglio_whale = "FJRZ5sTp27n6GhUVqgVkY4JGUJPjhRPnWtH4du5UhKbw";
    let a_whale = "MfDuWeqSHEqTFVYZ7LoexgAK9dxk7cy4DFJWjWMGVWa";
    let a_bad_whale = "bobCPc5nqVoX7r8gKzCMPLrKjFidjnSCrAdcYGCH2Ye";
    let a_magaiba_top_trader = "71WDyyCsZwyEYDV91Qrb212rdg6woCHYQhFnmZUBxiJ6";

    let log_program_ids = vec![
        a_magaiba_top_trader,
    ];

    let sub_logs_messages =
        json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "method": "logsSubscribe",
                        "params": [
                            {
                                "mentions": log_program_ids
                            },
                            {
                                "encoding": "jsonParsed",
                                "commitment": "finalized"
                            }
                        ]
                    }).to_string();

    println!("Subscribing to LOGS on {} with provided messages :: {:?}", solana_private_ws_url.clone(), sub_logs_messages.clone());
    solana_ws_stream.send(Message::Text(sub_logs_messages)).await?;

    //another way
    let sub_log_params = vec![
        ("logsSubscribe", vec!["5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1".to_string(), "finalized".to_string()]),
        // Add more subscriptions as needed
    ];
    // solana_subscriber.subscribe(&mut solana_ws_stream, &sub_log_params).await?;


    let (solana_event_sender, mut solana_event_receiver) =
        bounded::<SolanaEventTypes>(5000);

    let solana_ws_message_processing_task = tokio::spawn(async move {
        consume_stream::<SolanaEventTypes>(&mut solana_ws_stream, solana_event_sender).await;
    });

    //let new_token_tracker = Arc::new(Mutex::new(NewTokenTracker::new()));
    let solana_db_session_manager = db_session_manager.clone();
    let solana_private_http_url = env::var("PRIVATE_SOLANA_QUICKNODE_HTTP")
        .expect("PRIVATE_SOLANA_QUICKNODE_HTTP must be set");
    let client = Client::new();

    ///PROCESS DESERIALIZED SOLANA EVENTS
    let solana_task = tokio::spawn(async move {
        while let Ok(event) = solana_event_receiver.recv() {
            match event {
                SolanaEventTypes::LogNotification(ref log) => {
                    // println!("[[SOLANA TASK]] Processing log with signature {:?}", event);
                    if log.params.result.value.err.is_none() {
                        let signature = log.params.result.value.signature.clone();
                        // println!("[[SOLANA TASK]] SUCCESSFUL TRANSACTION Signature: {}", signature);

                        //get transaction with received signature
                        let transaction_response = client
                            .post(&solana_private_http_url)
                            .json(&json!({
                                "jsonrpc": "2.0",
                                "id": 1,
                                "method": "getTransaction",
                                "params": [
                                    signature,
                                    {
                                        "encoding": "jsonParsed",
                                        "maxSupportedTransactionVersion": 0
                                    }
                                ]
                            }))
                            .send()
                            .await;

                        if let Ok(response) = transaction_response {
                            if response.status().is_success() {
                                match response.text().await {
                                    Ok(text) => {
                                        let value: serde_json::Value = serde_json::from_str(&text).expect("Failed to deserialize into value !!!!");
                                        // println!("[[TRANSACTION DATA]]{:#?}", value);
                                        match serde_json::from_value::<RpcResponse>(value) {
                                            Ok(tx) => {
                                                if let Some(result) = tx.result {
                                                    let mut transfer_checked_instructions: Vec<TransferCheckedInfo> = Vec::new();
                                                    let pre: Vec<TokenBalance> = result.clone().meta.pre_token_balances;
                                                    let post: Vec<TokenBalance> = result.clone().meta.post_token_balances;

                                                    let inner_instructions = result.clone().meta.inner_instructions;
                                                    for inner_instruction in inner_instructions {
                                                        for instruction in inner_instruction.instructions {
                                                            if let Ok(transfer_checked) = serde_json::from_value::<TransferChecked>(instruction) {
                                                                if transfer_checked.parsed.as_ref().map_or(false, |p| p.instruction_type == "transferChecked") {
                                                                    // Found a TransferChecked instruction
                                                                    let transfer_info = transfer_checked.parsed.unwrap().info;
                                                                    // println!("[[TRANSFER CHECKED INFO]] {:?}", transfer_info);
                                                                    transfer_checked_instructions.push(transfer_info);
                                                                }
                                                            }
                                                        }
                                                    }

                                                     match prepare_transaction_summary(signature, pre, post, transfer_checked_instructions).await {
                                                         Ok(_) => {continue}
                                                         Err(_) => {continue}
                                                     }
                                                } else {
                                                    continue;
                                                }
                                            }
                                            Err(e) => eprintln!("Error deserializing transaction {:?}", e),
                                        }
                                    },
                                    Err(e) => eprintln!("Failed to read response text: {:?}", e),
                                }
                            } else {
                                eprintln!("Getting transaction returned failure: {:?}", response);
                            }
                        } else {
                            eprintln!("Could not get transaction for signature: {:?}", signature);
                        }
                    }
                }

                SolanaEventTypes::AccountNotification(notification) => {
                    let signature = notification;
                    println!("[[SOLANA TASK]] GOT ACCOUNT NOTIFICATION {:?}", signature)
                }
                _ => {
                    println!("Stand by")
                }
            }
        }
    });

    let _ = server::http_server::run_server().await;

    match tokio::try_join!(
        ws_server_task,
        solana_ws_message_processing_task,
        solana_task
    ) {
        Ok(_) => println!("All tasks completed successfully"),
        Err(e) => eprintln!("A task exited with an error: {:?}", e),
    }

    Ok(())
}

// https://developers.metaplex.com/token-metadata
pub struct Metadata {
    pub key: Key,
    pub update_authority: Pubkey,
    pub mint: Pubkey,
    pub data: Data,
    pub primary_sale_happened: bool,
    pub is_mutable: bool,
    // ...
}


async fn prepare_transaction_summary(
    signature: String,
    pre_token_balances: Vec<TokenBalance>,
    post_token_balances: Vec<TokenBalance>,
    transfer_checked_info: Vec<TransferCheckedInfo>
) -> Result<Vec<TxCheckedSummary>, Box<dyn Error>> {
    let mut summaries = Vec::new();
    let client = Client::new();
    let solana_private_http_url = env::var("PRIVATE_SOLANA_QUICKNODE_HTTP")
        .expect("PRIVATE_SOLANA_QUICKNODE_HTTP must be set");
    for transfer_info in transfer_checked_info {
        //track known addresses
        let mut known_addresses = HashMap::new();
        known_addresses.insert("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", ("USDC", "USD Coin"));
        known_addresses.insert("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", ("USDT", "USD Token"));


        let metadata_response = client
            .post(&solana_private_http_url)
            .json(&json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "getAccountInfo",
                "params": [
                    transfer_info.mint,
                    {
                        "encoding": "jsonParsed",
                    }
                ]
            }))
            .send()
            .await?;

        if metadata_response.status().is_success() {
            let response_text = metadata_response.text().await?;
            let value: serde_json::Value = serde_json::from_str(&response_text)?;
            println!("{:#?}", value);
        }

        // Consider retrieving actual token name and symbol using the mint address
        let summary = TxCheckedSummary {
            signature: signature.clone(),
            transaction_type: "Confirmed Transfer".to_string(),
            source: transfer_info.source,
            destination: transfer_info.destination,
            mint: transfer_info.mint,
            token_name: "Placeholder".to_string(),
            token_symbol: "Placeholder".to_string(),
            token_amount: Option::from(transfer_info.token_amount.ui_amount.unwrap_or_default()),
        };

        println!("{}", summary);
        summaries.push(summary);
    }

    Ok(summaries)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RpcResponse {
    id: u64,
    jsonrpc: String,
    result: Option<ResultField>,
    block_time: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResultField {
    meta: Meta,
    transaction: Transaction,
    slot: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    err: Option<Value>,
    fee: u64,
    inner_instructions: Vec<InnerInstruction>,
    post_balances: Vec<u64>,
    post_token_balances: Vec<TokenBalance>,
    pre_balances: Vec<u64>,
    pre_token_balances: Vec<TokenBalance>,
    rewards: Vec<Value>,
    status: Status,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalance {
    account_index: u64,
    mint: String,
    owner: String,
    program_id: String,
    ui_token_amount: UiTokenAmount,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiTokenAmount {
    amount: String,
    decimals: u8,
    ui_amount: Option<f64>,
    ui_amount_string: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InnerInstruction {
    index: u64,
    instructions: Vec<Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransferChecked {
    pub program: String,
    pub program_id: String,
    pub parsed: Option<TransferCheckedInstruction>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransferCheckedInstruction {
    #[serde(rename = "type")]
    pub instruction_type: String,
    pub info: TransferCheckedInfo,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransferCheckedInfo {
    pub authority: String,
    pub destination: String,
    pub mint: String,
    pub source: String,
    pub token_amount: TokenAmount,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenAmount {
    pub amount: String,
    pub decimals: u8,
    #[serde(rename = "uiAmount")]
    pub ui_amount: Option<f64>,
    #[serde(rename = "uiAmountString")]
    pub ui_amount_string: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Status {
    ok: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransactionMessage {
    account_keys: Vec<AccountKey>,
    instructions: Vec<Value>,
    recent_blockhash: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountKey {
    pubkey: String,
    signer: bool,
    writable: bool,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    message: TransactionMessage,
    signatures: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Header {
    num_readonly_signed_accounts: u8,
    num_readonly_unsigned_accounts: u8,
    num_required_signatures: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InstructionData {
    accounts: Vec<u8>,
    data: String,
    program_id_index: Option<u8>,
}