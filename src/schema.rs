
use diesel::table;
use serde_json::Value;

table! {
    polygon.polygon_crypto_level2_book_data {
        id -> Serial,
        event_type -> Varchar,
        pair -> Varchar,
        timestamp -> Bigint,
        received_timestamp -> Bigint,
        exchange_id -> Bigint,
        bid_prices -> Jsonb,
        ask_prices -> Jsonb,
    }
}

table! {
    polygon.polygon_crypto_aggregate_data {
        id -> Serial,
        event_type -> Varchar,
        pair -> Varchar,
        open -> Double,
        close -> Double,
        high -> Double,
        low -> Double,
        volume -> Double,
        timestamp -> Bigint,
        end_time -> Bigint,
        vw -> Double,
        avg_trade_size -> Bigint,
    }
}

table! {
    polygon.polygon_crypto_quote_data {
        id -> Serial,
        event_type -> Varchar,
        pair -> Varchar,
        bid_price -> Double,
        bid_size -> Double,
        ask_price -> Double,
        ask_size -> Double,
        timestamp -> Bigint,
        exchange_id -> Bigint,
        received_timestamp -> Bigint,
    }
}

table! {
    polygon.polygon_crypto_trade_data {
        id -> Serial,
        event_type -> Varchar,
        pair -> Varchar,
        price -> Double,
        timestamp -> Bigint,
        size -> Double,
        conditions -> Jsonb,
        trade_id -> Nullable<Varchar>,
        exchange_id -> Bigint,
        received_timestamp -> Bigint,
    }
}



// Define the Diesel models for each table
#[derive(Queryable)]
pub struct PolygonCryptoLevel2BookData {
    pub id: i32,
    pub event_type: String,
    pub pair: String,
    pub timestamp: i64,
    pub received_timestamp: i64,
    pub exchange_id: i64,
    pub bid_prices: Value,
    pub ask_prices: Value,
}

#[derive(Insertable)]
#[table_name = "polygon_crypto_level2_book_data"]
pub struct NewPolygonCryptoLevel2BookData {
    pub event_type: String,
    pub pair: String,
    pub timestamp: i64,
    pub received_timestamp: i64,
    pub exchange_id: i64,
    pub bid_prices: Value,
    pub ask_prices: Value,
}


#[derive(Queryable)]
pub struct PolygonCryptoAggregateData {
    pub id: i32,
    pub event_type: String,
    pub pair: String,
    pub open: f64,
    pub close: f64,
    pub high: f64,
    pub low: f64,
    pub volume: f64,
    pub timestamp: i64,
    pub end_time: i64,
    pub vw: f64,
    pub avg_trade_size: i64,
}

#[derive(Insertable)]
#[table_name = "polygon_crypto_aggregate_data"]
pub struct NewPolygonCryptoAggregateData {
    pub event_type: String,
    pub pair: String,
    pub open: f64,
    pub close: f64,
    pub high: f64,
    pub low: f64,
    pub volume: f64,
    pub timestamp: i64,
    pub end_time: i64,
    pub vw: f64,
    pub avg_trade_size: i64,
}

#[derive(Queryable)]
pub struct PolygonCryptoQuoteData {
    pub id: i32,
    pub event_type: String,
    pub pair: String,
    pub bid_price: f64,
    pub bid_size: f64,
    pub ask_price: f64,
    pub ask_size: f64,
    pub timestamp: i64,
    pub exchange_id: i64,
    pub received_timestamp: i64,
}

#[derive(Insertable)]
#[table_name = "polygon_crypto_quote_data"]
pub struct NewPolygonCryptoQuoteData {
    pub event_type: String,
    pub pair: String,
    pub bid_price: f64,
    pub bid_size: f64,
    pub ask_price: f64,
    pub ask_size: f64,
    pub timestamp: i64,
    pub exchange_id: i64,
    pub received_timestamp: i64,
}

#[derive(Queryable)]
pub struct PolygonCryptoTradeData {
    pub id: i32,
    pub event_type: String,
    pub pair: String,
    pub price: f64,
    pub timestamp: i64,
    pub size: f64,
    pub conditions: Value,
    pub trade_id: Option<String>,
    pub exchange_id: i64,
    pub received_timestamp: i64,
}

#[derive(Insertable)]
#[table_name = "polygon_crypto_trade_data"]
pub struct NewPolygonCryptoTradeData {
    pub event_type: String,
    pub pair: String,
    pub price: f64,
    pub timestamp: i64,
    pub size: f64,
    pub conditions: Value,
    pub trade_id: Option<String>,
    pub exchange_id: i64,
    pub received_timestamp: i64,
}

