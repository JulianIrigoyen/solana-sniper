use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use r2d2::{Error, PooledConnection};

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub struct DbSessionManager {
    pool: DbPool,
}

impl DbSessionManager {
    pub fn new(database_url: &str) -> Self {
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");
        DbSessionManager { pool }
    }

    pub fn get_connection(&self) -> Result<PooledConnection<ConnectionManager<PgConnection>>, Error> {
        self.pool.get()
    }


    // pub fn persist_event(&self, event: &PolygonEventTypes) -> Result<usize, diesel::result::Error> {
    //
    //     let mut conn = self.get_connection().unwrap(); // Assume this method fetches a DB connection
    //
    //     // Example conversion for one variant; implement for others as needed
    //     // let new_event = match event {
    //     //     PolygonEventTypes::XtTrade(trade_data) => {
    //     //         NewPolygonCryptoTradeData {
    //     //             event_type: "XT".to_string(),
    //     //             pair: trade_data.pair.clone(),
    //     //             price: trade_data.price,
    //     //             timestamp: trade_data.timestamp,
    //     //             size: trade_data.size,
    //     //             conditions: serde_json::Value::Array(Vec::new()),
    //     //             trade_id: trade_data.trade_id.clone(),
    //     //             exchange_id: trade_data.exchange_id,
    //     //             received_timestamp: trade_data.received_timestamp,
    //     //         }
    //     //     },
    //     //     // Handle other variants...
    //     //     _ => unimplemented!(),
    //     // };
    //
    //     // Insert the new event into the database
    //     // diesel::insert_into(polygon_crypto_trade_data)
    //     //     .values(&new_event)
    //     //     .execute(&mut conn)
    // }

}