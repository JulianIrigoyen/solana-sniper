use actix_web::{App, HttpServer, web};
use std::sync::Arc;

use crate::server::endpoints::transaction;
use crate::server::endpoints::signatures_for_address;
use crate::server::endpoints::holders;
use crate::server::endpoints::whales;
use crate::server::endpoints::new_spls;


pub async fn run_server() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(web::scope("/api")
                         // .configure(transaction::init_routes)
                         .configure(signatures_for_address::init_routes)
                         .configure(holders::init_routes)
                         .configure(whales::init_routes)
                         // .configure(new_spls::init_routes)
                         // .configure(analytics::init_routes) // Example for future modules
            )
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
