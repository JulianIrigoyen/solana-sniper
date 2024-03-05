// use std::hash::Hash;
// use crate::models::binance::binance_event_types::BinanceEventTypes;
// use crate::models::binance::diff_depth::DiffDepth;
// use crate::models::binance::kline::Kline;
// use crate::models::binance::partial_book_depth::PartialBookDepth;
// use crate::models::binance::trade::Trade;
// use std::collections::{BTreeMap, HashMap, VecDeque};
// use ordered_float::OrderedFloat;
//
//
// /**
//
// Responsibility: Monitor and analyze the order book depth data (bids and asks) for various symbols.
// Functionality:  Maintain a current state of the order book for each symbol, highlighting potential support and resistance levels based on the volume of bids and asks.
//
//
// Analysis:
//
//     Market Depth: Market depth refers to the volume of orders waiting to be executed at different price levels for a particular asset. It's visualized in what's often called an "order book". Depth data shows the demand (bids) and supply (asks) at different price points and the volume available at each level.
//         * Bids: Orders from buyers to purchase the asset at a certain price. They are listed in descending order with the highest bid at the top.
//         * Asks: Orders from sellers to sell the asset at a certain price. They are listed in ascending order with the lowest ask at the top.
//
//     Depth data is crucial because it provides insight into potential resistance (ASKS) and support (BIDS) levels.
//         * High volume at a bid level suggests strong buying interest that could act as support
//         * High volume at an ask level indicates selling interest that could act as resistance.
//
// */
// #[derive()]
// struct OrderLevel {
//     price: OrderedFloat<f64>,
//     quantity: f64,
// }
//
// struct OrderBookState {
//     bids: BTreeMap<OrderedFloat<f64>, OrderLevel>, // Sorted by price in descending order
//     asks: BTreeMap<OrderedFloat<f64>, OrderLevel>, // Sorted by price in ascending order
//     total_volume_history: VecDeque<(f64, f64)>, // (total bid volume, total ask volume)
// }
//
// impl OrderBookState {
//     //  OB State maintains the last 100 updates
//     pub const HISTORY_CAPACITY: usize = 100;
//
//     fn new() -> Self {
//         Self {
//             bids: BTreeMap::new(),
//             asks: BTreeMap::new(),
//             total_volume_history: VecDeque::with_capacity(Self::HISTORY_CAPACITY),
//         }
//     }
//
//     /// Order Book State Thresholds
//     /// Threshold for what constitutes a "rapid" volume change
//     const VOLUME_CHANGE_THRESHOLD_PERCENT: f64 = 2.0; // 10% change
//     fn analyze_and_alert(&mut self) {
//         self.detect_rapid_volume_change();
//         self.detect_disappearance_of_large_orders();
//         self.detect_sharp_bid_ask_spread_change();
//         self.detect_imbalance_between_bids_and_asks();
//     }
//
//     fn detect_rapid_volume_change(&mut self) {
//         // Calculate current total volume for bids and asks
//         let current_total_bid_volume: f64 = self.bids.values().map(|level| level.quantity).sum();
//         let current_total_ask_volume: f64 = self.asks.values().map(|level| level.quantity).sum();
//
//         // Compare against the most recent historical volume to detect rapid change
//         if let Some(&(last_bid_volume, last_ask_volume)) = self.total_volume_history.back() {
//             let bid_volume_change_percent = ((&current_total_bid_volume - last_bid_volume) / &last_bid_volume) * 100.0;
//             let ask_volume_change_percent = ((&current_total_ask_volume - last_ask_volume) / &last_ask_volume) * 100.0;
//
//             // Check if the change exceeds our threshold
//             if bid_volume_change_percent.abs() > Self::VOLUME_CHANGE_THRESHOLD_PERCENT || ask_volume_change_percent.abs() > Self::VOLUME_CHANGE_THRESHOLD_PERCENT {
//                 println!("Alert: Significant volume change detected. Bid change: {:.2}%, Ask change: {:.2}%", bid_volume_change_percent, ask_volume_change_percent);
//                 // Implement your alert mechanism here (e.g., logging, sending notifications)
//             }
//         }
//
//         if self.total_volume_history.len() == 100 { //history capacity
//             self.total_volume_history.pop_front(); // Remove the oldest record to make room
//         }
//         self.total_volume_history.push_back((current_total_bid_volume, current_total_ask_volume));
//     }
//
//     fn detect_disappearance_of_large_orders(&self) {
//         // Implement logic to compare current large orders against previous state
//         // Alert if large orders disappear without corresponding trades
//     }
//
//     fn detect_sharp_bid_ask_spread_change(&self) {
//         // Implement detection logic based on `bid_ask_spread_history`
//         // Alert if the spread changes significantly in a short period
//     }
//
//     fn detect_imbalance_between_bids_and_asks(&self) {
//         // Calculate total bid and ask volumes and compare for significant imbalance
//         // Alert if an imbalance is detected
//     }
//
//     // Additional methods for updating historical data...
// }
//
// pub struct DepthTracker {
//     pub order_books: HashMap<String, OrderBookState>
// }
//
// impl DepthTracker {
//     pub fn new() -> Self {
//         Self {
//             order_books: HashMap::new(),
//         }
//     }
//
//     pub(crate) fn apply(&mut self, event: &BinanceEventTypes) {
//         println!("APPLYING EVENT {:?}", event);
//         match event {
//             // BinanceEventTypes::Trade(data) => self.process_binance_trade(data),
//             BinanceEventTypes::PartialBookDepth(data) => self.update_partial_depth_data(data),
//             BinanceEventTypes::DiffDepth(data) => self.update_diff_depth_data(data),
//             _ => {}
//         }
//     }
//
//     /// The PartialBookDepth events provide snapshots of the top levels of the order book (both bids and asks) for a symbol. When such an event is received, the tracker should update the order book state for that symbol with the new top levels of bids and asks provided.
//     pub(crate) fn update_partial_depth_data(&mut self, depth: &PartialBookDepth) {
//         let order_book_state = self.order_books.entry("depth.symbol".to_string()).or_insert_with(OrderBookState::new);
//
//         // Clear existing bids and asks to replace with the new snapshot
//         order_book_state.bids.clear();
//         order_book_state.asks.clear();
//
//         // Update bids
//         for (price, quantity) in &depth.bids {
//             let price = price.parse::<f64>().expect("Invalid bid price");
//             let quantity = quantity.parse::<f64>().expect("Invalid bid quantity");
//             let ordered_float = ordered_float::OrderedFloat(price.clone());
//             order_book_state.bids.insert(ordered_float::OrderedFloat(price), OrderLevel {  price: ordered_float, quantity });
//         }
//
//         // Update asks
//         for (price, quantity) in &depth.asks {
//             let price = price.parse::<f64>().expect("Invalid ask price");
//             let quantity = quantity.parse::<f64>().expect("Invalid ask quantity");
//             let ordered_float = ordered_float::OrderedFloat(price.clone());
//             order_book_state.asks.insert(OrderedFloat(price), OrderLevel {  price: ordered_float, quantity });
//         }
//
//         // After updating the order book, analyze for volume changes
//         if let Some(order_book_state) = self.order_books.get_mut("depth.symbol") {
//             order_book_state.analyze_and_alert(); // This will call detect_rapid_volume_change among others
//         }
//
//     }
//
//     /// The DiffDepth events provide updates on changes to the order book for a symbol, including additions, updates, and deletions of orders at different price levels. When processing these events, the tracker should apply these differential changes to the existing state of the order book for the relevant symbol.
//     pub(crate) fn update_diff_depth_data(&mut self, depth: &DiffDepth) {
//         let order_book_state = self.order_books.entry(depth.symbol.clone()).or_insert_with(OrderBookState::new);
//
//         // Update bids
//         for (price, quantity) in &depth.bids {
//             let price = price.parse::<f64>().expect("Invalid bid price");
//             let quantity = quantity.parse::<f64>().expect("Invalid bid quantity");
//
//             if quantity == 0.0 {
//                 // If quantity is 0, remove the level
//                 order_book_state.bids.remove(&OrderedFloat(price));
//             } else {
//                 // Update or add the bid level
//                 let ordered_float = ordered_float::OrderedFloat(price.clone());
//                 order_book_state.bids.insert(OrderedFloat(price), OrderLevel {  price: ordered_float, quantity });
//             }
//         }
//
//         // Update asks
//         for (price, quantity) in &depth.asks {
//             let price = price.parse::<f64>().expect("Invalid ask price");
//             let quantity = quantity.parse::<f64>().expect("Invalid ask quantity");
//
//             if quantity == 0.0 {
//                 // If quantity is 0, remove the level
//                 order_book_state.asks.remove(&OrderedFloat(price));
//             } else {
//                 // Update or add the ask level
//                 let ordered_float = ordered_float::OrderedFloat(price.clone());
//                 order_book_state.asks.insert(ordered_float::OrderedFloat(price), OrderLevel {  price: ordered_float, quantity });
//             }
//         }
//
//         // After updating the order book, analyze for volume changes
//         if let Some(order_book_state) = self.order_books.get_mut(&depth.symbol) {
//             order_book_state.analyze_and_alert(); // This includes detect_rapid_volume_change
//         }
//     }
// }
//
