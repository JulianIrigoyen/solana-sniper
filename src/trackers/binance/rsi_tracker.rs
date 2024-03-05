// use std::collections::HashMap;
// use lazy_static::lazy_static;
// use crate::models::binance::binance_event_types::BinanceEventTypes;
// use crate::models::binance::diff_depth::DiffDepth;
// use crate::models::binance::kline::Kline;
// use crate::models::binance::partial_book_depth::PartialBookDepth;
// use crate::models::binance::trade::Trade;
//
// /**
//     The purpose of this struct is to calculate RSI values for a given symbol over different intervals and aggregate depth data for insight.
//
//     Market Depth: Market depth refers to the volume of orders waiting to be executed at different price levels for a particular asset. It's visualized in what's often called an "order book". Depth data shows the demand (bids) and supply (asks) at different price points and the volume available at each level.
//         Bids: Orders from buyers to purchase the asset at a certain price. They are listed in descending order with the highest bid at the top.
//         Asks: Orders from sellers to sell the asset at a certain price. They are listed in ascending order with the lowest ask at the top.
//
//     Depth data is crucial because it provides insight into potential resistance (in the case of asks) and support (in the case of bids) levels.
//     - High volume at a bid level suggests strong buying interest that could act as support
//     - High volume at an ask level indicates selling interest that could act as resistance.
//
//     Leveraging Depth Data and RSI for Smart Alerts
//
//     * Identifying Overbought/Oversold Conditions: Use RSI to identify potential overbought (>70) or oversold (<30) conditions. These conditions suggest that an asset might be due for a reversal.
//     * Analyze Depth Data for Confirmation: Once an overbought or oversold condition is detected, examine the depth data for confirmation.
//         For example, in an oversold condition (RSI < 30), look for a significant volume of bids just below the current price, indicating potential support.
//         Conversely, in an overbought condition (RSI > 70), look for a substantial volume of asks just above the current price, indicating potential resistance.
//
//     * Consider Volume Imbalance: A significant imbalance between bids and asks can indicate the direction of potential price movement. A high volume of bids compared to asks might indicate upward pressure on price, while the opposite suggests downward pressure.
//
//     * Generate alerts when both RSI indicators and depth data align. For example:
//
//         ! Buy Alert: If RSI is below 30 (oversold) and there's significant bid volume just below the current price, indicating strong support.
//         ! Sell Alert: If RSI is above 70 (overbought) and there's significant ask volume just above the current price, indicating strong resistance.
//
// */
//
//
// lazy_static! {
//     static ref INTERVAL_PERIODS: HashMap<&'static str, usize> = {
//         let mut m = HashMap::new();
//         m.insert("1d", 14);
//         m.insert("15m", 14);
//         m.insert("5min", 14);
//         m.insert("1s", 14);
//         m
//     };
// }
//
//
// // TODO for this to work properly, we need a process that collects relevant historical data - we need a starting point RSI for each of the symbol intervals being tracked.
// pub struct RsiTracker {
//     interval_rsis: HashMap<String, f64>,
//     interval_price_changes: HashMap<String, Vec<f64>>,
//     pub(crate) symbol_intervals: HashMap<String, Vec<String>>,
// }
//
// impl RsiTracker {
//     pub fn new() -> Self {
//         Self {
//             interval_rsis: HashMap::new(),
//             interval_price_changes: HashMap::new(),
//             symbol_intervals: HashMap::new(),
//         }
//     }
//
//     /// Retrieves the RSI value for the given symbol and interval.
//     pub fn get_rsi(&self, symbol: &str, interval: &str) -> Option<f64> {
//         let key = format!("{}_{}", symbol, interval);
//         // Use the get method of the HashMap to retrieve the RSI value for the given symbol and interval.
//         // The get method returns an Option, which will be None if the key is not found.
//         self.interval_rsis.get(&key).copied()
//     }
//     /// Sets the initial RSI value for the given symbol and interval.
//     pub fn set_initial_rsi(&mut self, symbol: &str, interval: &str, initial_rsi: f64) {
//         let key = format!("{}_{}", symbol, interval);
//         self.interval_rsis.insert(key, initial_rsi);
//     }
//
//     /// Registers the intervals for which RSI will be tracked for a given symbol (1min, 15min, 1 day, etc...)
//     pub fn set_intervals_for_symbol(&mut self, symbol: &str, intervals: Vec<&str>) {
//         let intervals = intervals.into_iter().map(String::from).collect();
//         self.symbol_intervals.insert(symbol.to_string(), intervals);
//         println!("[[RSI TRACKER]] Intervals set. Current intervals: {:?}", self.symbol_intervals);
//     }
//
//     // Applies a kline to the tracker, updating price changes and calculating RSI
//     pub fn apply_kline(&mut self, kline: &Kline) {
//         let key = format!("{}_{}", kline.symbol, kline.kline.interval);
//
//         // Check if the symbol's interval is being tracked
//         if let Some(intervals) = self.symbol_intervals.get(&kline.symbol) {
//             if intervals.contains(&kline.kline.interval) {
//                 println!(
//                     "APPLYING KLINE Symbol: {}, Close Price: {}, Volume: {}, Number of Trades: {}",
//                     &key, kline.kline.close_price, kline.kline.volume, kline.kline.number_of_trades
//                 );
//
//                 let close_price: f64 = match kline.kline.close_price.parse() {
//                     Ok(price) => price,
//                     Err(err) => {
//                         eprintln!("Error parsing close price: {:?}", err);
//                         return; // Return early if parsing fails
//                     }
//                 };
//                 // Pass the period as a parameter
//                 let period = INTERVAL_PERIODS.get(&*kline.kline.interval).unwrap().clone();
//                 self.update_price_change(&kline.symbol, &kline.kline.interval, close_price, period);
//                 self.calculate_rsi(&key);
//             } else {
//                 println!("Interval '{}' not found for symbol '{}'", kline.kline.interval, kline.symbol);
//             }
//         } else {
//             println!("Symbol '{}' not found in tracked symbols", kline.symbol);
//         }
//     }
//
//
//     /// Updates the price change for the specified symbol and interval.
//     fn update_price_change(&mut self, symbol: &str, interval: &str, new_price: f64, period: usize) {
//         let key = format!("{}_{}", symbol, interval);
//
//         let price_changes = self.interval_price_changes.entry(key.clone()).or_insert_with(Vec::new);
//
//         if let Some(&last_price) = price_changes.last() {
//             let price_change = new_price - last_price;
//             price_changes.push(price_change);
//         }
//         else {
//             // Calculate the initial price change (assuming a zero initial price)
//             let initial_price = 51751.15; // Adjust this based on your initial price assumption
//             let price_change = new_price - initial_price;
//             price_changes.push(price_change);
//         }
//
//         if price_changes.len() > period + 1 {
//             price_changes.remove(0);
//         }
//     }
//
//     /// Calculates the Relative Strength Index (RSI) for the specified symbol and interval.
//     fn calculate_rsi(&mut self, key: &str) {
//         // Retrieve price changes for the specified symbol and interval
//         let pc = self.interval_price_changes.get(key);
//         println!("CALCULATING RSI - Current price changes for {}, {:?}", key, pc);
//         if let Some(price_changes) = self.interval_price_changes.get(key) {
//             // Check if there are enough price changes to calculate RSI
//             if price_changes.len() <= 1 {
//                 println!("CALCULATING RSI - Not enough data points for {}, {:?}", key, pc);
//                 return; // Not enough data points, return early
//             }
//
//             // Accumulate sum of gains, sum of losses, count of gains, and count of losses
//             let (sum_gains, sum_losses, count_gains, count_losses) = price_changes
//                 // Iterate over pairs of adjacent elements in the price_changes slice using the `windows` method
//                 .windows(2).fold(
//                 (0.0, 0.0, 0, 0), // Initial tuple with sum_gains, sum_losses, count_gains, and count_losses
//                 |(sum_gains, sum_losses, count_gains, count_losses), window| {
//                     // Calculate price change between adjacent elements
//                     let change = window[1].clone() - window[0].clone(); // Calculate price change
//                     // Update accumulated values based on price change
//                     if change > 0.0 {
//                         (sum_gains + change, sum_losses, count_gains + 1, count_losses)
//                     } else {
//                         (sum_gains, sum_losses - change, count_gains, count_losses + 1)
//                     }
//                 },
//             );
//
//             // Calculate average gain and average loss
//             let average_gain = if count_gains > 0 { sum_gains / count_gains as f64 } else { 0.0 };
//             let average_loss = if count_losses > 0 { sum_losses / count_losses as f64 } else { 0.0 };
//
//             // Calculate Relative Strength (RS)
//             let rs = if average_loss == 0.0 {
//                 if average_gain == 0.0 {
//                     1.0 // Avoid division by zero, set to 1.0
//                 } else {
//                     f64::INFINITY // Considered overbought, set to infinity
//                 }
//             } else {
//                 average_gain / average_loss // Normal RS calculation
//             };
//
//             // Calculate RSI using RS
//             let rsi = 100.0 - (100.0 / (1.0 + rs));
//
//             // Insert calculated RSI into the interval RSI map
//             println!("[[RSI TRACKER]] CALCULATED RSI FOR'{}' :::  {}", key, &rsi);
//             self.interval_rsis.insert(key.to_string(), rsi);
//         }
//     }
// }
