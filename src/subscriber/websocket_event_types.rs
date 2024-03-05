use std::error::Error;
use serde_json::Value;
use crate::models::solana::solana_event_types::SolanaEventTypes;
use crate::util::serde_helper::deserialize_into;

///Trait used for event deserialization
pub trait WebsocketEventTypes: Sized {
    // Returns a descriptive name or type of the event.
    fn event_type(&self) -> String;

    // Method to deserialize a JSON value into a specific event type.
    fn deserialize_event(value: &Value) -> Result<Self, Box<dyn Error>>;
}

impl WebsocketEventTypes for SolanaEventTypes {
    fn event_type(&self) -> String {
        match self {
            SolanaEventTypes::LogNotification(_) => "LogNotification".to_string(),
            SolanaEventTypes::ProgramNotification(_) => "ProgramNotification".to_string(),
        }
    }

    fn deserialize_event(value: &Value) -> Result<SolanaEventTypes, Box<dyn Error>> {
        println!("{}", format!("Attempting to deserialize SOLANA event:: {:?}", value));

        let result = match value {
            _ => {
                Err("Deserialization error: unsupported type or malformed JSON".into())
            }
        };
        match &result {
            Ok(deserialized) => println!("BINANCE EVENT: {:?}", deserialized),
            Err(e) => {}
        }

        result
    }
}
