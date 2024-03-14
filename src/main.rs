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

//deserializer for data received from solana_client
use borsh::BorshDeserialize;
use std::{env, thread};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::io;

use std::error::Error as StdError;


use crossbeam_channel::{bounded, Sender};

use dotenv::dotenv;

use reqwest::Client as Client;
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use timely::dataflow::InputHandle;
use timely::dataflow::operators::{Filter, Input, Inspect};
use timely::execute_from_args;
use timely::worker::Worker;

use tokio::time::{interval as tokio_interval, Duration as TokioDuration, Instant};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, WebSocketStream};
use tokio_tungstenite::tungstenite::{Error as WsError, Message};
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::MaybeTlsStream;
use tokio::sync::Mutex;

use futures_util::stream::{Stream, StreamExt};
use futures_util::{future, pin_mut, SinkExt};
use futures::task::{Context, Poll};
use futures::Future;

use log::{info, warn, error};
use url::Url;

use crate::db::db_session_manager::DbSessionManager;
use crate::models::solana::solana_event_types::SolanaEventTypes;
use crate::models::solana::alchemy::get_program_accounts::ProgramAccountsResponse;
use crate::server::ws_server;
use crate::subscriber::websocket_subscriber::{WebSocketSubscriber, AuthMethod, SolanaSubscriptionBuilder};
use crate::subscriber::consume_stream::{consume_stream};
use crate::trackers::raydium::new_token_tracker;
use crate::trackers::raydium::new_token_tracker::NewTokenTracker;

use actix::prelude::*;
use actix_web::web;
use mpl_token_metadata::accounts::Metadata;
use mpl_token_metadata::ID;

// retry impl
use stream_reconnect::{UnderlyingStream, ReconnectStream, ReconnectOptions};

use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use tokio::time;
use tokio::time::interval;
use crate::models::solana::solana_account_notification::SolanaAccountNotification;
use crate::models::solana::solana_transaction::{TransactionSummary, TxCheckedSummary};
use crate::scraper::birdeye_scraper::scrape_wallet_addresses;
use crate::server::endpoints::birdeye::token_prices::{fetch_multi_token_prices, MultiPriceRequest};

mod db;
mod util;
mod models;
mod server;
mod http;
mod trackers;
mod schema;
mod subscriber;
mod decoder;
mod scraper;

/** Welcome to the Solana Sniper */

/// Custom Implementation of SolanaReconnectStream - https://lib.rs/crates/stream-reconnect
pub struct SolanaWebSocket(WebSocketStream<MaybeTlsStream<TcpStream>>);

impl UnderlyingStream<String, Result<Message, WsError>, WsError> for SolanaWebSocket {
    // Establishes connection.
    // Additionally, this will be used when reconnect tries are attempted.
    fn establish(addr: String) -> Pin<Box<dyn Future<Output = Result<Self, WsError>> + Send>> {
        Box::pin(async move {
            match connect_async(&addr).await {
                Ok((stream, _)) => Ok(SolanaWebSocket(stream)),
                Err(e) => Err(e),
            }
        })
    }

    // The following errors are considered disconnect errors.
    fn is_write_disconnect_error(&self, err: &WsError) -> bool {
        matches!(
                err,
                WsError::ConnectionClosed
                    | WsError::AlreadyClosed
                    | WsError::Io(_)
                    | WsError::Tls(_)
                    | WsError::Protocol(_)
            )
    }

    // If an `Err` is read, then there might be an disconnection.
    fn is_read_disconnect_error(&self, item: &Result<Message, WsError>) -> bool {
        if let Err(e) = item {
            Self::is_write_disconnect_error(&self, e)
        } else {
            false
        }
    }

    // Return "Exhausted" if all retry attempts are failed.
    fn exhaust_err() -> WsError {
        WsError::Io(io::Error::new(io::ErrorKind::Other, "Exhausted"))
    }
}

type ReconnectWs = ReconnectStream<SolanaWebSocket, String, Result<Message, WsError>, WsError>;

impl Stream for SolanaWebSocket {
    type Item = Result<Message, WsError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        Pin::new(&mut this.0).poll_next(cx)
    }
}

use futures::Sink;

impl Sink<Message> for SolanaWebSocket {
    type Error = WsError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0).poll_ready(cx)
    }

    fn start_send(mut self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        Pin::new(&mut self.0).start_send(item)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0).poll_close(cx)
    }

}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // let wallets = scrape_wallet_addresses().await?; TODO birdeye blocked </3
    // Setup shared state for last activity tracking
    let last_activity = Arc::new(AtomicU64::new(Instant::now().elapsed().as_secs()));


    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_session_manager = Arc::new(DbSessionManager::new(&database_url));
    // let solana_db_session_manager = db_session_manager.clone(); TODO use this to start storing data

    // ------------ WEBSOCKET SERVER INITIALIZATION ------------
    let ws_host = env::var("WS_SERVER_HOST").expect("WS_HOST must be set");
    let ws_port = env::var("WS_SERVER_PORT").expect("WS_PORT must be set");
    let ws_server_task = tokio::spawn(async move {
        let ws_server = ws_server::WebSocketServer::new(ws_host, ws_port);
        //TODO use whenever we want to serve data
        // ws_server.run().await
    });

    // ------------ WEBSOCKET CONNECTION ------------
    let solana_public_ws_url    = String::from("wss://api.mainnet-beta.solana.com");
    let solana_private_ws_url   = env::var("PRIVATE_SOLANA_QUICKNODE_WS").expect("PRIVATE_SOLANA_QUICKNODE_WS must be set");
    let solana_private_http_url = env::var("PRIVATE_SOLANA_QUICKNODE_HTTP").expect("PRIVATE_SOLANA_QUICKNODE_HTTP must be set");

    let active_ws_url = solana_public_ws_url.clone();


    //TODO replace if moving away from stream-reconnect
    // let (mut solana_ws_stream, _) = connect_async(active_ws_url.clone()).await?;

    //https://solana.com/docs/rpc/websocket/accountsubscribe
    // * api key is provided in the path
    let solana_subscriber = WebSocketSubscriber::<SolanaSubscriptionBuilder>::new(
        solana_private_ws_url.to_string(),
        // solana_public_ws_url.to_string(), //TODO here to switch between private and public RPC URLS
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

    // println!("Subscribing to PROGRAM NOTIFICATIONS on  {} with provided messages :: {:?}", solana_private_ws_url.clone(), sub_accounts_message.clone());
    // solana_ws_stream.send(Message::Text(sub_accounts_message)).await?;

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

    // println!("Subscribing to ACCOUNT NOTIFICATOINS on  {} with provided messages :: {:?}", solana_private_ws_url.clone(), sub_accounts_message.clone());
    // solana_ws_stream.send(Message::Text(sub_accounts_message)).await?;

    // ------------ LOG SUBSCRIPTION ------------
    let raydium_public_key = "srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX";
    let openbook_public_key = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";

    // --- WHALE KEYS --- https://birdeye.so/leaderboard/7D?chain=solana
    let miglio_whale = "FJRZ5sTp27n6GhUVqgVkY4JGUJPjhRPnWtH4du5UhKbw";
    let a_whale = "MfDuWeqSHEqTFVYZ7LoexgAK9dxk7cy4DFJWjWMGVWa";
    let a_bad_whale = "bobCPc5nqVoX7r8gKzCMPLrKjFidjnSCrAdcYGCH2Ye";
    let a_magaiba_top_trader = "71WDyyCsZwyEYDV91Qrb212rdg6woCHYQhFnmZUBxiJ6";
    let a_solana_top_trader = "MfDuWeqSHEqTFVYZ7LoexgAK9dxk7cy4DFJWjWMGVWa";
    let a_solana_top_trader_2 = "DzYV9AFEbe9eGc8GRaNvsGjnt7coYiLDY7omCS1jykJU";

    //TODO THIS IS USED BELOW TO BUILD TRANSACTION SUMMARIES
    let currently_tracked_whale = a_magaiba_top_trader.clone();

    let log_program_ids = vec![
        currently_tracked_whale.clone(),
    ];

    let sub_logs_message =
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

    println!("Subscribing to LOG NOTIFICATIONS on {} with provided messages :: {:?}", active_ws_url.clone(), sub_logs_message.clone());
    let reconnect_options = ReconnectOptions::new()
        .with_retries_generator(|| {
            std::iter::repeat_with(|| Duration::from_secs(5))
        });
    let mut reconnect_stream: ReconnectStream<SolanaWebSocket, String, Result<Message, WsError>, WsError> =
        ReconnectStream::connect_with_options(active_ws_url.clone(), reconnect_options).await?;

    // solana_ws_stream.send(Message::Text(sub_logs_message)).await?;
    reconnect_stream.send(Message::Text(sub_logs_message)).await?;


    // ------------ CHANNEL CREATION ------------
    let (solana_event_sender, mut solana_event_receiver) =
        bounded::<SolanaEventTypes>(5000);

        let sender_clone = solana_event_sender.clone();

    let solana_ws_message_processing_task = tokio::spawn(async move {
        consume_stream::<SolanaEventTypes>(&mut reconnect_stream, solana_event_sender).await;
    });

    let solana_private_http_url = env::var("PRIVATE_SOLANA_QUICKNODE_HTTP")
        .expect("PRIVATE_SOLANA_QUICKNODE_HTTP must be set");

    let client = Client::new();

    // ------------ DESERIALIZED SOLANA EVENT PROCESSING ------------
    let solana_task = tokio::spawn(async move {
        while let Ok(event) = solana_event_receiver.recv() {
            match event {
                SolanaEventTypes::LogNotification(ref log) => {
                    // println!("[[SOLANA TASK]] Processing log with signature {:?}", event);
                    if log.params.result.value.err.is_none() {
                        let signature = log.params.result.value.signature.clone();
                        // println!("[[SOLANA TASK]] SUCCESSFUL TRANSACTION Signature: {}", signature);

                        // ------------ GET TRANSACTION WITH RECEIVED SIGNATURE ------------
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

                        // ------------ PROCESS TRANSACTION INSTRUCTION AND PRE/POST TOKEN BALANCES ------------
                        // For now, we are only supporting transactions of type TRANSFER CHECKED. We can easily implement a parser for any type following this exmaple.
                        if let Ok(response) = transaction_response {
                            if response.status().is_success() {
                                match response.text().await {
                                    Ok(text) => {
                                        let value: serde_json::Value = serde_json::from_str(&text).expect("Failed to deserialize into value !!!!");
                                        // println!("[[TRANSACTION DATA]] {:#?}", value);
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
                                                                if let Some(parsed) = transfer_checked.parsed {
                                                                    if parsed.instruction_type == "transferChecked" {
                                                                        // Found a TransferChecked instruction
                                                                        let transfer_info = parsed.info;
                                                                        // println!("[[TRANSFER CHECKED INFO]] {:?}", transfer_info);
                                                                        transfer_checked_instructions.push(transfer_info);
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }

                                                     match prepare_transaction_summary_2(
                                                         signature.clone(),
                                                         currently_tracked_whale.to_string(),
                                                         pre,
                                                         post).await {

                                                         Ok(_) => {
                                                             println!("Successfully processed transaction {:?}", signature.clone());
                                                             continue
                                                         }
                                                         Err(e) => {
                                                             println!("Failed to process transaction {:?} : {:?}", signature.clone(), e);
                                                             continue
                                                         }
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
                    println!("Stand by");
                    continue;
                }
            }
        }
    });
    // println!("Spawning heartbeat task...");
    // let do_heartbeats = tokio::spawn( async move {
    //     heartbeat(heartbeat_stream, last_activity.clone()).await
    // });
    let _ = server::http_server::run_server().await;

    // let do_heartbeats = tokio::spawn( async move {
    //     heartbeat(heartbeat_stream, last_activity.clone()).await
    // });

    match tokio::try_join!(
        ws_server_task,
        solana_ws_message_processing_task,
        solana_task,
        // do_heartbeats //TODO impl recovery
    ) {
        Ok(_) => println!("All tasks completed successfully"),
        Err(e) => eprintln!("A task exited with an error: {:?}", e),
    }
    Ok(())
}

// https://developers.metaplex.com/token-metadata


async fn prepare_transaction_summary_2(
    signature: String,
    tracked_whale: String,
    pre_token_balances: Vec<TokenBalance>,
    post_token_balances: Vec<TokenBalance>,
) -> Result<Vec<TxCheckedSummary>, Box<dyn std::error::Error>> {
    let solana_private_http_url = env::var("PRIVATE_SOLANA_QUICKNODE_HTTP").expect("PRIVATE_SOLANA_QUICKNODE_HTTP must be set");
    let client = RpcClient::new(solana_private_http_url);

    let balance_changes = aggregate_balance_changes(pre_token_balances.clone(), post_token_balances.clone(), &tracked_whale).await?;
    // e.g., changes must be greater than or equal to 1.0 tokens (trying to avoid micro transactions for 0.0000....n USDT/USDC/SOL
    let significance_threshold = 1.0;

    let significant_changes: Vec<&BalanceChange> = balance_changes.iter()
        .filter(|change| change.change.abs() >= significance_threshold)
        .collect();

    if significant_changes.is_empty() {
        // Handle case with no significant changes if necessary
        return Ok(vec![]);
    }

    let mut summaries: Vec<TxCheckedSummary> = vec![];

    for change in significant_changes.iter() {
        let token_name: String;
        let token_symbol: String;

        // Fetch token metadata
        match fetch_token_metadata(&client, &change.mint).await {
            Ok(metadata) => {
                token_name = metadata.0;
                token_symbol = metadata.1;
            }
            Err(e) => {
                eprintln!("Error while fetching token metadata: {:?}", e);
                token_name = "Unknown".to_string();
                token_symbol = "Unknown".to_string();
            }
        }

        let transaction_type = if change.change < 0.0 { "Sale" } else { "Purchase" };
        let description = format!(
            "{} {} {:.2} {} ({})",
            tracked_whale,
            transaction_type,
            change.change.abs(),
            token_name,
            token_symbol
        );

        // fetch token prices
        let list_address: String = significant_changes.iter()
            .map(|change| change.mint.clone())
            .collect::<Vec<String>>().join(",");

        let price_request = web::Json(MultiPriceRequest { list_address });


        let token_prices = fetch_multi_token_prices(price_request).await?;

        let mut summary = TxCheckedSummary {
            signature: signature.clone(),
            transaction_type: transaction_type.to_string(),
            source: "N/A".to_string(),
            destination: "N/A".to_string(),
            mint: change.mint.clone(),
            token_name,
            token_symbol,
            token_amount: Some(change.change.abs()),
            token_price: None,
            transaction_usd_value: None,
            price_change_24_h: None,
            liquidity: None,
            detail: description,
        };
        if let Some(token_data) = token_prices.iter().find(|data| data.program_id == change.mint) {
            // Update the summary object with the price and liquidity
            summary.token_price = Some(token_data.clone().value);
            summary.liquidity = Some(token_data.clone().liquidity);
            summary.price_change_24_h = Some(token_data.clone().price_change_24_h);
        }

        println!("{}", summary);
        println!("------------------------------------------------------------");
        println!("------------------------------------------------------------");
        summaries.push(summary);
    }

    Ok(summaries)
}

async fn fetch_token_metadata(client: &RpcClient, mint: &String) ->  Result<(String, String), Box<dyn std::error::Error>> {
let metadata_program_id = &ID;
let token_mint_address = Pubkey::from_str(mint.as_str()).unwrap();
let (metadata_account_address, _) = Pubkey::find_program_address(
    &[
        b"metadata",
        metadata_program_id.as_ref(),
        token_mint_address.as_ref(),
    ],
    &metadata_program_id,
);


let account_data = client.get_account_data(&metadata_account_address)?;
let metadata = Metadata::from_bytes(&account_data)
    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

Ok((metadata.name, metadata.symbol))
}

// Function to aggregate balance changes between pre and post token balances
async fn aggregate_balance_changes(
    pre_balances: Vec<TokenBalance>,
    post_balances: Vec<TokenBalance>,
    whale: &str
) -> Result<Vec<BalanceChange>, Box<dyn std::error::Error>> {
    let mut changes: Vec<BalanceChange> = Vec::new();
    let mut pre_map: HashMap<String, f64> = HashMap::new();
    let mut post_map: HashMap<String, f64> = HashMap::new();

    // Populate the pre-map with pre-transaction balances
    for balance in pre_balances.into_iter().filter(|b| &b.owner == whale) {
        pre_map.insert(balance.mint.clone(), balance.ui_token_amount.ui_amount.unwrap_or_default());
    }

    // Populate the post-map and calculate the change
    for balance in post_balances.into_iter().filter(|b| &b.owner == whale) {
        post_map.insert(balance.mint.clone(), balance.ui_token_amount.ui_amount.unwrap_or_default());
    }

    // Determine the change in balance for each token and categorize as purchase or sale
    for (mint, post_amount) in post_map.iter() {
        let pre_amount = pre_map.get(mint).copied().unwrap_or(0.0);
        let change = post_amount - pre_amount;
        if change != 0.0 {
            changes.push(BalanceChange {
                mint: mint.clone(),
                change,
            });
        }
    }

    Ok(changes)
}


// Structure to hold changes in token balances
#[derive(Debug, Serialize, Deserialize, Clone)]
struct BalanceChange {
    mint: String,
    change: f64, // Positive for net purchases, negative for net sales.
}

fn extract_amount(token_balance: &TokenBalance) -> Result<f64, &'static str> {
    match token_balance.clone().ui_token_amount.ui_amount {
        Some(amount) => Ok(amount),
        None => Err("Amount not available"),
    }
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

/*
async fn prepare_transaction_summary(
    signature: String,
    tracked_whale: String,
    pre_token_balances: Vec<TokenBalance>,
    post_token_balances: Vec<TokenBalance>,
    transfer_checked_info: Vec<TransferCheckedInfo>
) -> Result<Vec<TxCheckedSummary>, Box<dyn Error>> {
    let mut summaries = Vec::new();
    let solana_private_http_url = env::var("PRIVATE_SOLANA_QUICKNODE_HTTP")
        .expect("PRIVATE_SOLANA_QUICKNODE_HTTP must be set");
    let client = RpcClient::new(solana_private_http_url);

    //track known addresses
    let mut known_addresses = HashMap::new();
    known_addresses.insert("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", ("USDC", "USD Coin"));
    known_addresses.insert("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", ("USDT", "USD Token"));
    known_addresses.insert("So11111111111111111111111111111111111111112", ("SOL", "Wrapped SOL"));

    // Determine buy or sell
    let mut activity_detail = String::new();

    let pre_balance = pre_token_balances.iter()
        .find(|balance| balance.owner == tracked_whale);
    let post_balance = post_token_balances.iter()
        .find(|balance| balance.owner == tracked_whale);

    if let (Some(pre), Some(post)) = (pre_balance, post_balance) {
        let pre_amount = pre.ui_token_amount.ui_amount.unwrap_or(0.0);
        let post_amount = post.ui_token_amount.ui_amount.unwrap_or(0.0);

        if post_amount > pre_amount {
            activity_detail = format!("{} bought {:.2} tokens", tracked_whale, post_amount - pre_amount);
        } else if pre_amount > post_amount {
            activity_detail = format!("{} sold {:.2} tokens", tracked_whale, pre_amount - post_amount);
        }
    }

    // println!("{:?}", activity_detail);

    for transfer_info in transfer_checked_info {

        if known_addresses.contains_key(&*transfer_info.mint) {
            // If the mint is known, skip fetching metadata and printing summary
            //todo suele ser ruido / internal transfers
            continue;
        }

        let metadata_program_id = &ID;
        let token_mint_address = Pubkey::from_str(transfer_info.mint.as_str()).unwrap();
        let (metadata_account_address, _) = Pubkey::find_program_address(
            &[
                b"metadata",
                metadata_program_id.as_ref(),
                token_mint_address.as_ref(),
            ],
            &metadata_program_id,
        );

        // Attempt to fetch and deserialize the account data for the metadata account
        let account_data_result = client.get_account_data(&metadata_account_address);
        let (token_name, token_symbol) = match account_data_result {
            Ok(account_data) => match Metadata::from_bytes(&account_data) {
                Ok(metadata) => {
                    // println!("[[METADATA]] {:?}", metadata);
                    (metadata.name, metadata.symbol)
                },
                Err(e) => {
                    eprintln!("Error while parsing metadata: {:?}", e);
                    // Default to "Unknown" if there's an error parsing metadata
                    ("Unknown".to_string(), "Unknown".to_string())
                }
            },
            Err(e) => {
                eprintln!("Failed to fetch account data: {:?}", e);
                // Same
                ("Unknown".to_string(), "Unknown".to_string())
            }
        };

        let summary = TxCheckedSummary {
            signature: signature.clone(),
            transaction_type: "Confirmed Transfer".to_string(),
            source: transfer_info.source.clone(),
            destination: transfer_info.destination.clone(),
            mint: transfer_info.mint.clone(),
            token_name,
            token_symbol,
            token_amount: Some(transfer_info.token_amount.ui_amount.unwrap_or(0.0)),
            detail: activity_detail.clone()
        };

        println!("{}", summary);
        println!("------------------------------------------------------------");
        println!("------------------------------------------------------------");
        summaries.push(summary);
    }

    Ok(summaries)
}
*/