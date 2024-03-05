use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};
use tokio::time::{interval, Duration};

pub struct WebSocketServer {
    host: String,
    port: String,
}

impl WebSocketServer {
    pub fn new(host: String, port: String) -> Self {
        WebSocketServer { host, port }
    }

    pub async fn run(&self) {
        let addr = format!("{}:{}", self.host, self.port);
        let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
        println!("WebSocket server listening on: {}", addr);

        let connection_id = Arc::new(Mutex::new(0u64)); // Counter for assigning IDs to connections

        while let Ok((stream, _)) = listener.accept().await {
            let id = {
                let mut lock = connection_id.lock().await;
                *lock += 1;
                *lock
            };

            println!("New connection: {}", id);
            tokio::spawn(handle_connection(stream, id));
        }
    }
}

async fn handle_connection(stream: TcpStream, id: u64) {
    println!("Handling connection {}", id);
    let ws_stream = accept_async(stream).await.expect("Error during the websocket handshake");

    // Split the WebSocket stream into a sender and receiver
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Create a channel for sending "heartbeat" messages
    let (tx, mut rx) = mpsc::channel(100);

    // Spawn a task to read from the channel and send messages
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            ws_sender.send(msg).await.expect("Failed to send message");
        }
    });

    // Spawn a task to send a heartbeat message every 10 seconds
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            println!("Sending heartbeat to connection {}", id);
            if tx.send(Message::Text(format!("heartbeat to {}", id))).await.is_err() {
                println!("Connection {} closed, stopping heartbeat.", id);
                break;
            }
        }
    });

    // Optionally handle incoming messages
    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(msg) => println!("Received a message from connection {}: {:?}", id, msg),
            Err(e) => {
                eprintln!("Error receiving message from connection {}: {:?}", id, e);
                break;
            }
        }
    }
}