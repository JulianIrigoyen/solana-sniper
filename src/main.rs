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
use std::sync::{Arc, Mutex};
use std::task::Context;

use crossbeam_channel::{bounded, Sender};
use dotenv::dotenv;
use futures_util::{sink::SinkExt, stream::StreamExt};
use reqwest::Client;
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
use crate::models::solana::solana_transaction::SolanaTransaction;
use crate::models::solana::solana_account_notification::SolanaAccountNotification;

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


// Supervisor actor that spawns signature processors or manages a pool of them

// Message to instruct the supervisor to process a signature
#[derive(Message)]
#[rtype(result = "()")]
struct ProcessSignature {
    signature: String,
}

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
        // ws_server.run().await
    });

    let solana_public_ws_url = String::from("wss://api.mainnet-beta.solana.com");
    let solana_private_ws_url = env::var("PRIVATE_SOLANA_QUICKNODE_WS").expect("PRIVATE_SOLANA_QUICKNODE_WS must be set");
    let client = Client::new();
    let solana_private_http_url = env::var("PRIVATE_SOLANA_QUICKNODE_HTTP").expect("PRIVATE_SOLANA_QUICKNODE_HTTP must be set");



    // api key is provided in the path
    // let solana_api_key = env::var("ALCHEMY_SOLANA_API_KEY").expect("ALCHEMY_SOLANA_API_KEY must be set");
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

    let subscription_program_ids = vec![
        "EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm",      // WIF
        "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",     // BONK
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
            }).to_string(),
            ],
        );
        sub_program_params.push(param);
    }

    let account_program_ids: Vec<String> = vec![
        ("FJRZ5sTp27n6GhUVqgVkY4JGUJPjhRPnWtH4du5UhKbw".to_string()), //whale de miglio
        ("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1".to_string()), //raydium
        ("11111111111111111111111111111111".to_string()), //system program
        ("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string()), //token program
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


    let raydium_public_key = "srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX";
    let miglio_whale = "FJRZ5sTp27n6GhUVqgVkY4JGUJPjhRPnWtH4du5UhKbw";
    let a_whale = "MfDuWeqSHEqTFVYZ7LoexgAK9dxk7cy4DFJWjWMGVWa";
    let openbook_public_key = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";
    let log_program_ids = vec![
        raydium_public_key,
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

    let new_token_tracker = Arc::new(Mutex::new(NewTokenTracker::new()));
    let solana_db_session_manager = db_session_manager.clone();

    let solana_task = tokio::spawn(async move {
        while let Ok(event) = solana_event_receiver.recv() {
            // println!("[[SOLANA TASK]] got event {:?}", event);
            match event {
                SolanaEventTypes::LogNotification(ref log) => {

                    // println!("[[SOLANA TASK]] Processing log with signature {:?}", event);
                    if log.params.result.value.err.is_none() {
                        // If there are no errors, print the event and extract the signature
                        let signature = log.params.result.value.signature.clone();
                        println!("[[SOLANA TASK]] SUCCESSFUL TRANSACTION Signature: {}", signature);
                        // Here's where you make the HTTP request
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
                                match response.text().await {  // Properly await and match the result
                                    Ok(text) => {  // `text` is of type `String` here
                                        // Now you can use `text` as a `String`
                                        match serde_json::from_str::<serde_json::Value>(&text) {
                                            Ok(value) => println!("[[SOLANA TASK]] FOUND TRANSACTION: {:#?}", value),
                                            Err(e) => eprintln!("Failed to deserialize transaction: {:?}", e),
                                        }
                                    },
                                    Err(e) => eprintln!("Failed to read response text: {:?}", e),
                                }
                            } else {
                                // If the response status is not successful, log the status
                                eprintln!("Error fetching transaction details: {:?}", response.status());
                            }
                        } else {
                            eprintln!("Failed to send the request or receive the response");
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


    match tokio::try_join!(ws_server_task, solana_ws_message_processing_task, solana_task) {
        Ok(_) => println!("All tasks completed successfully"),
        Err(e) => eprintln!("A task exited with an error: {:?}", e),
    }
    


    Ok(())
}

async fn fetch_transaction_details(signature: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    // Implement the logic to fetch transaction details using the signature
    // This will likely involve making an HTTP request to the Solana JSON RPC API
    println!("GETTING TX FOR {:?}", signature);
    Ok(serde_json::Value::Null) // Placeholder
}
//
// async fn fetch_token_metadata(account: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
//     // Implement the logic to fetch token metadata
//     // This might involve querying the Metaplex token metadata program, for example
//     Ok(serde_json::Value::Null) // Placeholder
// }
