use crate::models::solana::solana_logs_notification::SolanaLogsNotification;
use crate::models::solana::solana_program_notification::SolanaProgramNotification;
use crate::models::solana::solana_account_notification::SolanaAccountNotification;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SolanaEventTypes {
    LogNotification(SolanaLogsNotification),
    AccountNotification(SolanaAccountNotification),
    ProgramNotification(SolanaProgramNotification)
}
