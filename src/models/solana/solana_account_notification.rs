// use serde_json::Value;
// use std::convert::TryFrom;
// use std::error::Error;
// use crate::models::solana::solana_event_types::SolanaEventTypes;
//
// // https://solana.com/docs/rpc/websocket/accountsubscribe
// // DataField, NotificationData, ParsedData, ParsedInfo, and FeeCalculator definitions remain the same
//
// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct SolanaAccountNotification {
//     pub jsonrpc: String,
//     pub method: String,
//     pub params: NotificationParams,
// }
//
// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct NotificationParams {
//     pub result: NotificationResult,
//     pub subscription: u64,
// }
//
// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct NotificationResult {
//     pub context: NotificationContext,
//     pub value: NotificationValue,
// }
//
// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct NotificationContext {
//     pub slot: u64,
// }
//
// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct NotificationValue {
//     pub executable: bool,
//     pub lamports: u64,
//     pub owner: String,
//     pub rent_epoch: u64,
//     pub space: u64,
//     pub data: DataField,
//
// }
//
// #[derive(Serialize, Deserialize, Debug, Clone)]
// #[serde(untagged)]
// enum DataField {
//     Base64(Vec<String>), // base64 data is an array with two strings as seen in the error messages we got
//     ParsedData(NotificationData),
// }
//
// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct NotificationData {
//     pub program: String,
//     pub parsed: ParsedData,
// }
//
// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct ParsedData {
//     pub r#type: String, // Using raw identifier syntax for reserved keywords
//     pub info: ParsedInfo,
// }
//
// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct ParsedInfo {
//     pub authority: String,
//     pub blockhash: String,
//     pub fee_calculator: FeeCalculator,
// }
//
// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct FeeCalculator {
//     pub lamports_per_signature: u64,
// }
//
// impl Default for SolanaAccountNotification {
//     fn default() -> Self {
//         SolanaAccountNotification {
//             jsonrpc: "".to_string(),
//             method: "".to_string(),
//             params: NotificationParams::default(),
//         }
//     }
// }
//
// impl Default for NotificationParams {
//     fn default() -> Self {
//         NotificationParams {
//             result: NotificationResult::default(),
//             subscription: 0,
//         }
//     }
// }
//
// impl Default for NotificationResult {
//     fn default() -> Self {
//         NotificationResult {
//             context: NotificationContext::default(),
//             value: NotificationValue::default(),
//         }
//     }
// }
//
// impl Default for NotificationContext {
//     fn default() -> Self {
//         NotificationContext {
//             slot: 0,
//         }
//     }
// }
//
// impl Default for NotificationValue {
//     fn default() -> Self {
//         NotificationValue {
//             data: NotificationData::default(),
//             executable: false,
//             lamports: 0,
//             owner: "".to_string(),
//             rent_epoch: 0,
//             space: 0,
//         }
//     }
// }
//
// impl Default for NotificationData {
//     fn default() -> Self {
//         NotificationData {
//             program: "".to_string(),
//             parsed: ParsedData::default(),
//         }
//     }
// }
//
// impl Default for ParsedData {
//     fn default() -> Self {
//         ParsedData {
//             r#type: "".to_string(),
//             info: ParsedInfo::default(),
//         }
//     }
// }
//
// impl Default for ParsedInfo {
//     fn default() -> Self {
//         ParsedInfo {
//             authority: "".to_string(),
//             blockhash: "".to_string(),
//             fee_calculator: FeeCalculator::default(),
//         }
//     }
// }
//
// impl Default for FeeCalculator {
//     fn default() -> Self {
//         FeeCalculator {
//             lamports_per_signature: 0,
//         }
//     }
// }
