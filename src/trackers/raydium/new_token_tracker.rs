use std::hash::Hash;
use crate::models::solana::solana_event_types::SolanaEventTypes;
use std::collections::{BTreeMap, HashMap, VecDeque};
use crate::models::solana::solana_logs_notification::SolanaLogsNotification;


/*struct TrackerState {
    new_tokens: HashMap<String, String>, // token tx signature / data
}

impl TrackerState {
    //  OB State maintains the last 1000 new tokens
    pub const HISTORY_CAPACITY: usize = 100;

    fn new() -> Self {
        Self {
            new_tokens: HashMap::new(),
        }
    }

    /// Order Book State Thresholds
    /// Threshold for what constitutes a "rapid" volume change
    const VOLUME_CHANGE_THRESHOLD_PERCENT: f64 = 2.0; // 10% change
    fn analyze_and_alert(&mut self) {
        self.detect_rapid_volume_change();
        self.detect_disappearance_of_large_orders();
        self.detect_sharp_bid_ask_spread_change();
        self.detect_imbalance_between_bids_and_asks();
    }

    fn detect_rapid_volume_change(&mut self) {
        // Calculate current total volume for bids and asks
        let current_total_bid_volume: f64 = self.bids.values().map(|level| level.quantity).sum();
        let current_total_ask_volume: f64 = self.asks.values().map(|level| level.quantity).sum();

        // Compare against the most recent historical volume to detect rapid change
        if let Some(&(last_bid_volume, last_ask_volume)) = self.total_volume_history.back() {
            let bid_volume_change_percent = ((&current_total_bid_volume - last_bid_volume) / &last_bid_volume) * 100.0;
            let ask_volume_change_percent = ((&current_total_ask_volume - last_ask_volume) / &last_ask_volume) * 100.0;

            // Check if the change exceeds our threshold
            if bid_volume_change_percent.abs() > Self::VOLUME_CHANGE_THRESHOLD_PERCENT || ask_volume_change_percent.abs() > Self::VOLUME_CHANGE_THRESHOLD_PERCENT {
                println!("Alert: Significant volume change detected. Bid change: {:.2}%, Ask change: {:.2}%", bid_volume_change_percent, ask_volume_change_percent);
                // Implement your alert mechanism here (e.g., logging, sending notifications)
            }
        }

        if self.total_volume_history.len() == 100 { //history capacity
            self.total_volume_history.pop_front(); // Remove the oldest record to make room
        }
        self.total_volume_history.push_back((current_total_bid_volume, current_total_ask_volume));
    }

    fn detect_disappearance_of_large_orders(&self) {
        // Implement logic to compare current large orders against previous state
        // Alert if large orders disappear without corresponding trades
    }

    fn detect_sharp_bid_ask_spread_change(&self) {
        // Implement detection logic based on `bid_ask_spread_history`
        // Alert if the spread changes significantly in a short period
    }

    fn detect_imbalance_between_bids_and_asks(&self) {
        // Calculate total bid and ask volumes and compare for significant imbalance
        // Alert if an imbalance is detected
    }

    // Additional methods for updating historical data...
}
*/

pub struct NewTokenTracker {
    pub new_tokens: HashMap<String, String>, // token tx signature / data
}

impl NewTokenTracker {
    pub fn new() -> Self {
        Self {
            new_tokens: HashMap::new(),
        }
    }

    pub(crate) fn apply(&mut self, event: &SolanaEventTypes) {
        println!("APPLYING NEW TOKEN TRACKER EVENT {:?}", event);
        match event {
            SolanaEventTypes::LogNotification(log) => self.handle_new_token_signature(log),
            _ => {}
        }
    }

    /// The PartialBookDepth events provide snapshots of the top levels of the order book (both bids and asks) for a symbol. When such an event is received, the tracker should update the order book state for that symbol with the new top levels of bids and asks provided.
    pub(crate) fn handle_new_token_signature(&mut self, log: &SolanaLogsNotification) {
        println!("[[NEW TOKEN TRACKER]] Processing signature: {:?}", log.params.result.value.signature)

    }
}

