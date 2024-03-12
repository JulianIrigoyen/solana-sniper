use std::error::Error;
use serde_json::Value;
use crate::models::solana::solana_event_types::SolanaEventTypes;
use crate::models::solana::solana_logs_notification::SolanaLogsNotification;
use crate::models::solana::solana_account_notification::SolanaAccountNotification;
use crate::models::solana::solana_program_notification::SolanaProgramNotification;
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
            SolanaEventTypes::AccountNotification(_) =>"AccountNotification".to_string()
        }
    }

    fn deserialize_event(value: &Value) -> Result<Self, Box<dyn Error>> {
        // println!("{}", format!("Attempting to deserialize SOLANA event:: {:?}", value));

        let method = value["method"].as_str().ok_or_else(|| "Missing method in event")?;
        let result = match method {
            "logsNotification" => {
                let log_notification = deserialize_into::<SolanaLogsNotification>(value)?;
                // println!("Deserialized log for TX with signature: {}", log_notification.params.result.value.signature);
                Ok(SolanaEventTypes::LogNotification(log_notification))
            },

            "accountNotification" => {
                let account_notification = deserialize_into::<SolanaAccountNotification>(value)?;
                println!("Signature: {}", account_notification);
                Ok(SolanaEventTypes::AccountNotification(account_notification))
            },

            "programNotification" => {
                deserialize_into::<SolanaProgramNotification>(&value)
                    .map(SolanaEventTypes::ProgramNotification)
            },
            _ => Err(format!("Unsupported event type: {}", method).into()),
        };

        match &result {
            Ok(deserialized) => println!("[[DESERIALIZER]] DESERIALIZED SOLANA EVENT: {:?}", deserialized),
            Err(e) => println!("Error deserializing Solana event: {:?}", e),
        }

        result
    }

}
