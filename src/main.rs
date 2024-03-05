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

mod db;
mod util;
mod models;
mod server;
mod http;
mod trackers;
mod schema;
mod subscriber;

/**

Welcome to the Solana Sniper.

 */

// fn run_filtered_binance_dataflow(
//     event: BinanceEventTypes,
//     worker: &mut Worker<timely::communication::Allocator>,
//     filters: EventFilters,
//     db_session_manager: Arc<DbSessionManager>,
//     rsi_tracker: Arc<Mutex<RsiTracker>>,
//     depth_tracker: Arc<Mutex<DepthTracker>>,
// ) {
//     worker.dataflow(|scope| {
//         let (mut input_handle, stream) = scope.new_input::<BinanceEventTypes>();
//
//         stream.inspect(move |event| {
//
//
//             let rsi_tracker_clone = Arc::clone(&rsi_tracker);
//             let depth_tacker_clone = Arc::clone(&depth_tracker);
//             // if let BinanceEventTypes::Kline(kline) = event {
//             //     if kline.symbol == "BTCUSDT" {
//             //         match kline.kline.interval.as_str() {
//             //             "1s" | "5m" | "15m" => {
//             //                 // Lock the RsiTracker for each event to safely update its state
//             //                 let mut tracker = rsi_tracker_clone.lock().unwrap();
//             //                 tracker.apply_kline(kline);
//             //
//             //                 if let Some(rsi) = tracker.get_rsi(&kline.symbol, &kline.kline.interval) {
//             //                     println!("Updated RSI for {} at {} interval: {}", kline.symbol, kline.kline.interval, rsi);
//             //                 }
//             //             },
//             //             _ => {} // Ignore other intervals
//             //         }
//             //     }
//             // }
//
//             if let BinanceEventTypes::PartialBookDepth(depth) = event {
//                 let mut tracker = depth_tracker.lock().unwrap();
//                 tracker.apply(event);
//             }
//         });
//
//         input_handle.send(event);
//         input_handle.advance_to(1);
//     });
// }
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_session_manager = Arc::new(DbSessionManager::new(&database_url));

    // Websocket Server Initialization
    let ws_host = env::var("WS_SERVER_HOST").expect("WS_HOST must be set");
    let ws_port = env::var("WS_SERVER_PORT").expect("WS_PORT must be set");
    let ws_server_task = tokio::spawn(async move {
        let ws_server = ws_server::WebSocketServer::new(ws_host, ws_port);
        ws_server.run().await
    });

    let solana_ws_url = String::from("wss://api.mainnet-beta.solana.com");

    // api key is provided in the path
    // let solana_api_key = env::var("ALCHEMY_SOLANA_API_KEY").expect("ALCHEMY_SOLANA_API_KEY must be set");
    let (mut solana_ws_stream, _) = connect_async(solana_ws_url.clone()).await?;
    println!("Connected to Solana WebSocket");

    let solana_subscriber = WebSocketSubscriber::<SolanaSubscriptionBuilder>::new(
        solana_ws_url.to_string(),
        None,
        AuthMethod::None,
        SolanaSubscriptionBuilder,
    );

    /** Program Ids
                     - WIF:          EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm
                     - RETARDIO:     6ogzHhzdrQr9Pgv6hZ2MNze7UrzBMAFyBBWUYp1Fhitx
                     - BONK:         DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263
                     - UPDOG:        HJ39rRZ6ys22KdB3USxDgNsL7RKiQmsC3yL8AS3Suuku
     */

    let program_ids = vec![
        ("EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm", "base64"), // WIF
        ("6ogzHhzdrQr9Pgv6hZ2MNze7UrzBMAFyBBWUYp1Fhitx", "jsonParsed"), // RETARDIO
        ("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263", "base64"), // BONK
        ("HJ39rRZ6ys22KdB3USxDgNsL7RKiQmsC3yL8AS3Suuku", "jsonParsed"), // UPDOG
    ];

    let mut params: Vec<(&str, Vec<String>)> = Vec::new();

    for (id, encoding) in program_ids {
        let param = (
            "programSubscribe",
            vec![
                id.to_string(),
                json!({
                "encoding": encoding,
                "commitment": "finalized"
            }).to_string(),
            ],
        );
        params.push(param);
    }

    let log_params = vec![
        ("logsSubscribe", vec!["6ogzHhzdrQr9Pgv6hZ2MNze7UrzBMAFyBBWUYp1Fhitx".to_string(), "finalized".to_string()]),
        // Add more subscriptions as needed
    ];

    // Subscribe to solana streams
    solana_subscriber.subscribe(&mut solana_ws_stream, &log_params).await?;

    let (solana_event_sender, solana_event_receiver) =
        bounded::<SolanaEventTypes>(5000);

    let solana_ws_message_processing_task = tokio::spawn(async move {
        // consume_stream::<SolanaEventTypes>(&mut solana_ws_stream, solana_event_sender).await;
    });


    // Wait for all tasks to complete
    let _ = tokio::try_join!(
        ws_server_task,
        // solana_ws_message_processing_task
    );

    let _ = server::http_server::run_server().await;


    Ok(())
}