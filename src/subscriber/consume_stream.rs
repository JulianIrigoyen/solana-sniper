use async_trait::async_trait;
use crossbeam_channel::Sender;
use futures_util::StreamExt;
use serde_json::Value;
use std::error::Error as StdError;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream, MaybeTlsStream};
use tungstenite::Error;

use crate::subscriber::websocket_event_types::WebsocketEventTypes;


pub async fn consume_stream<T: WebsocketEventTypes + Send + 'static>(
    ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    tx: Sender<T>,
) {
    while let Some(message) = ws_stream.next().await {
        // println!("Got message message: {:?}", message);
        match message {

            Ok(Message::Text(text)) => {
                // if text.contains("initialize2") {
                // println!("[[CONSUM STREAM]] GOT MESSAGE: {}", text);
                if let Err(e) = process_text_message(text, &tx).await {
                    // eprintln!("Failed to process text message: {:?}", e);
                }
                // }


            }
            Err(e) => eprintln!("Error receiving message: {:?}", e),
            _ => {}
        }
    }
}

async fn process_text_message<T: WebsocketEventTypes + Send + 'static>(
    text: String,
    tx: &Sender<T>,
) -> Result<(), Box<dyn StdError>> {
    let event_jsons: Result<Value, _> = serde_json::from_str(&text);
    match event_jsons {
        Ok(events) => {
            process_json_events(events, tx)?;
        }
        Err(e) => {
            eprintln!("Error parsing JSON: {:?}", e);
        }
    }
    Ok(())
}

fn process_json_events<T: WebsocketEventTypes + Send + 'static>(
    events: Value,
    tx: &Sender<T>,
) -> Result<(), Box<dyn StdError>> {
    if events.is_array() {
        for event in events.as_array().unwrap() {
            process_single_event(event, tx)?;
        }
    } else {
        process_single_event(&events, tx)?;
    }
    Ok(())
}

fn process_single_event<T: WebsocketEventTypes + Send + 'static>(
    event: &Value,
    sender: &Sender<T>,
) -> Result<(), Box<dyn StdError>> {
    match T::deserialize_event(event) {
        Ok(event) => {
            sender.send(event).map_err(|e| e.into())
        }
        Err(e) => {
            // eprintln!("consume_stream.process_single_event: Error deserializing message: {:?}", e);
            Err(e.into())
        }
    }
}

