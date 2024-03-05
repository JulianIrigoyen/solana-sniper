use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SolanaLogsNotification {
    pub jsonrpc: String,
    pub method: String,
    pub params: NotificationParams,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NotificationParams {
    pub result: NotificationResult,
    pub subscription: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NotificationResult {
    pub context: NotificationContext,
    pub value: NotificationValue,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NotificationContext {
    pub slot: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NotificationValue {
    pub signature: String,
    pub err: Option<serde_json::Value>,
    pub logs: Vec<String>,
}

impl Default for SolanaLogsNotification {
    fn default() -> Self {
        SolanaLogsNotification {
            jsonrpc: "".to_string(),
            method: "".to_string(),
            params: NotificationParams::default(),
        }
    }
}

impl Default for NotificationParams {
    fn default() -> Self {
        NotificationParams {
            result: NotificationResult::default(),
            subscription: 0,
        }
    }
}

impl Default for NotificationResult {
    fn default() -> Self {
        NotificationResult {
            context: NotificationContext::default(),
            value: NotificationValue::default(),
        }
    }
}

impl Default for NotificationContext {
    fn default() -> Self {
        NotificationContext {
            slot: 0,
        }
    }
}

impl Default for NotificationValue {
    fn default() -> Self {
        NotificationValue {
            signature: "".to_string(),
            err: None,
            logs: vec![],
        }
    }
}
