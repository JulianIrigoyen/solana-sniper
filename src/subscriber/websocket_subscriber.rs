use std::error::Error;

use futures_util::SinkExt;
use futures_util::StreamExt;
use serde_json::json;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, tungstenite::protocol::Message, WebSocketStream};
use url::Url;

pub trait SubscriptionBuilder {
    fn build_subscription_messages(params: &[(&str, Vec<String>)]) -> Vec<Message>;
}

pub enum AuthMethod {
    None,
    QueryParam,
    Message,
}

pub struct WebSocketSubscriber<B: SubscriptionBuilder> {
    ws_url: String,
    api_key: Option<String>,
    auth_method: AuthMethod,
    builder: B,
}

impl<B: SubscriptionBuilder> WebSocketSubscriber<B> {
    pub fn new(ws_url: String, api_key: Option<String>, auth_method: AuthMethod, builder: B) -> Self {
        Self { ws_url, api_key, auth_method, builder }
    }

    pub async fn connect(&self) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn Error>> {

        // println!("CONNECTING {:?}", self.ws_url);
        let final_url = match self.auth_method {
            AuthMethod::QueryParam => {
                if let Some(ref key) = self.api_key {
                    format!("{}{}", self.ws_url, key)
                } else {
                    self.ws_url.clone()
                }
            }
            _ => self.ws_url.clone(),
        };

        let url = Url::parse(&final_url)?;
        let (mut ws_stream, _) = connect_async(url).await?;
        println!("Connected to WebSocket :: {}", final_url);

        if let AuthMethod::Message = self.auth_method {
            self.authenticate(&mut ws_stream).await?;
        }

        Ok(ws_stream)
    }

    async fn authenticate(&self, ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>) -> Result<(), Box<dyn Error>> {
        if let Some(ref api_key) = self.api_key {
            let auth_message = Message::Text(json!({
                "action": "auth",
                "params": api_key
            }).to_string());

            ws_stream.send(auth_message).await?;
            println!("Authentication message sent");
        }

        Ok(())
    }


    pub async fn subscribe(&self, ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>, params: &[(&str, Vec<String>)]) -> Result<(), Box<dyn Error>> {
        let messages = B::build_subscription_messages(params);
        for message in messages {
            println!("Subscribing to {} with provided messages :: {:?}", self.ws_url, message);
            ws_stream.send(message).await?;
        }

        Ok(())
    }
}

pub struct SolanaSubscriptionBuilder;

impl SubscriptionBuilder for SolanaSubscriptionBuilder {
    fn build_subscription_messages(params: &[(&str, Vec<String>)]) -> Vec<Message> {
        params.iter().map(|&(method, ref args)| {
            let message = match method {
                "accountSubscribe" => {
                    let pubkey = args.get(0).expect("Account pubkey is required");
                    json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "method": method,
                        "params": [
                            pubkey,
                            {
                                "encoding": "jsonParsed",
                                "commitment": "finalized"
                            }
                        ]
                    })
                }
                "logsSubscribe" => {
                    if args.is_empty() || args[0] == "all" {
                        // Subscribe to all transactions except for simple vote transactions
                        json!({
                            "jsonrpc": "2.0",
                            "id": 1,
                            "method": "logsSubscribe",
                            "params": ["all"]
                        })
                    } else if args[0] == "allWithVotes" {
                        // Subscribe to all transactions, including simple vote transactions
                        json!({
                            "jsonrpc": "2.0",
                            "id": 1,
                            "method": "logsSubscribe",
                            "params": ["allWithVotes"]
                        })
                    } else {
                        println!("[[SUBSCRIBER]] SUBSCRIBING TO LOGS");
                        json!({
                            "jsonrpc": "2.0",
                            "id": 1,
                            "method": "logsSubscribe",
                            "params": [
                                {
                                    "mentions": [args[0]]
                                },
                                {
                                    "commitment": "finalized"
                                }
                            ]
                        })
                    }
                }

                "programSubscribe" => {
                    let program_id = args.get(0).expect("Program ID is required");
                    let encoding = "jsonParsed";
                    let commitment = "finalized";

                    let filters = args.iter().skip(3).map(|filter| {
                        serde_json::from_str::<serde_json::Value>(filter)
                            .expect("Filter must be a valid JSON")
                    }).collect::<Vec<_>>();

                    json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "method": method,
                        "params": [
                            program_id,
                            {
                                "encoding": encoding,
                                "commitment": "finalized",
                                "filters": filters
                            }
                        ]
                    })
                }
                _ => panic!("Unsupported subscription method: {}", method),
            };
            Message::Text(message.to_string())
        }).collect()
    }
}

