// use std::collections::HashMap;
// use std::sync::Arc;
//
// // use crate::models::polygon::polygon_crypto_aggregate_data::PolygonCryptoAggregateData;
// use serde::{Deserialize, Serialize};
//
//
// /// This class structure illustrates a highly modular and scalable approach to filtering real-time financial data streams.
//
// /// Enums with associated data (like Number and Text) allow for the representation of filter criteria values that can be of different types (e.g., numeric or textual), showcasing Rust's algebraic data types.
// /// This enables the creation of flexible filters that can apply to various data fields.
// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub enum FilterValue {
//     Number(f64),
//     Text(String),
// }
//
// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct FilterCriteria {
//     pub field: String,
//     pub operation: String,
//     pub value: FilterValue,
// }
//
// /**
//     The ParameterizedFilter structure in Rust is a powerful and flexible mechanism for applying custom filters to data streams,
//         particularly useful in the context of processing real-time financial data streams
//
//     Custom Criteria Mapping:
//         The ParameterizedFilter struct uses a HashMap to associate cryptocurrency pairs with a vector of FilterCriteria.
//             This mapping allows the application of multiple, distinct filtering criteria to different cryptocurrency pairs.
//             It enables users to specify conditions under which a trade or aggregate data point should be considered relevant or ignored.
//
//     Dynamic Criterion Evaluation:
//         The meets_criterion method dynamically evaluates whether a given trade meets the specified criteria.
//             This evaluation is based on the trade's attributes, such as price and size, and the criterion's operation (e.g., greater than, less than, equal to). This dynamic evaluation supports various operations and makes the filter highly adaptable to different filtering needs.
//
//     Generic Filter Application:
//         Through the implementation of the FilterFunction trait, ParameterizedFilter provides a generic apply method.
//             This method determines applicability of the filter to any event within the data stream, enabling seamless integration into the data processing pipeline. It checks if the event matches the filter's criteria and applies the filter accordingly.
//
//  */
// pub struct ParameterizedFilter {
//     criteria_by_pair: HashMap<String, Vec<FilterCriteria>>,
// }
//
// impl ParameterizedFilter {
//     pub fn new(criteria_by_pair: HashMap<String, Vec<FilterCriteria>>) -> Self {
//         ParameterizedFilter { criteria_by_pair }
//     }
//     // fn meets_criterion(&self, trade: &PolygonCryptoTradeData, criterion: &FilterCriteria) -> bool {
//     //     match &criterion.value {
//     //         FilterValue::Number(num_val) => match criterion.field.as_str() {
//     //             "price" => {
//     //                 self.compare_numeric(trade.clone().price, &criterion.operation, num_val.clone())
//     //             }
//     //             "size" => {
//     //                 self.compare_numeric(trade.clone().size, &criterion.operation, num_val.clone())
//     //             }
//     //             _ => false,
//     //         },
//     //         FilterValue::Text(text_val) => match criterion.field.as_str() {
//     //             "pair" => self.compare_text(&trade.pair, &criterion.operation, text_val),
//     //             _ => false,
//     //         },
//     //     }
//     // }
//
//     fn compare_numeric(&self, field_value: f64, operation: &str, criterion_value: f64) -> bool {
//         match operation {
//             ">" => field_value > criterion_value,
//             "<" => field_value < criterion_value,
//             "=" => (field_value - criterion_value).abs() < f64::EPSILON,
//             _ => false,
//         }
//     }
//
//     fn compare_text(&self, field_value: &str, operation: &str, criterion_value: &str) -> bool {
//         match operation {
//             "=" => field_value == criterion_value,
//             "!=" => field_value != criterion_value,
//             _ => false,
//         }
//     }
// }
//
// /// Any struct implementing this trait must provide an apply method, enabling a consistent way to apply filters to data events.
// pub trait FilterFunction {
//     fn apply(&self, event: &PolygonEventTypes) -> bool;
// }
// //
// // impl FilterFunction for ParameterizedFilter {
// //     fn apply(&self, event: &PolygonEventTypes) -> bool {
// //         match event {
// //             PolygonEventTypes::XtTrade(trade) => {
// //                 if let Some(criteria) = self.criteria_by_pair.get(&trade.pair) {
// //                     criteria
// //                         .iter()
// //                         .all(|criterion| self.meets_criterion(trade, criterion))
// //                 } else {
// //                     true // If no criteria for the pair, pass the trade through
// //                 }
// //             }
// //             _ => false,
// //         }
// //     }
// // }
//
// pub struct PriceMovementFilter {
//     threshold_percentage: f64,
// }
//
// // impl PriceMovementFilter {
// //     pub fn new(threshold_percentage: f64) -> Self {
// //         PriceMovementFilter {
// //             threshold_percentage,
// //         }
// //     }
// //
// //     fn apply_to_aggregate(&self, aggregate: &PolygonCryptoAggregateData) -> bool {
// //         let price_movement = (aggregate.close - aggregate.open).abs();
// //         let percentage_movement = (price_movement / aggregate.open) * 100.0;
// //         percentage_movement > self.threshold_percentage
// //     }
// // }
//
// impl FilterFunction for PriceMovementFilter {
//     fn apply(&self, event: &PolygonEventTypes) -> bool {
//         match event {
//             PolygonEventTypes::XaAggregateMinute(aggregate) => self.apply_to_aggregate(aggregate),
//             _ => false, // This filter does not apply to other event types
//         }
//     }
// }
//
// /**
//      The EventFilters sruct serves as a container for a collection of filters that are applied to incoming data streams.
//         Each filter is encapsulated within an Arc<dyn FilterFunction>, allowing for shared ownership across threads and dynamic dispatch of the apply method defined by the FilterFunction trait.
// */
//
// pub struct EventFilters {
//     pub(crate) filters: Vec<Arc<dyn FilterFunction>>,
// }
//
// impl EventFilters {
//     ///Creates a new instance of EventFilters with an empty vector of filters. Inits the filter system before any data processing begins.
//     pub fn new() -> Self {
//         Self {
//             filters: Vec::new(),
//         }
//     }
//
//     ///Allows adding new filters to the EventFilters instance. By taking an Arc<dyn FilterFunction>, it supports adding filters that implement the FilterFunction trait, enabling polymorphism. This design choice allows for different types of filters (e.g., price movement, trade size) to be applied without changing the underlying system architecture.
//     pub fn add_filter(&mut self, filter: Arc<dyn FilterFunction>) {
//         self.filters.push(filter);
//     }
// }
