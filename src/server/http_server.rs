use actix_web::{App, HttpServer, web};
use std::sync::Arc;

use crate::server::endpoints::accounts;
use crate::server::endpoints::transactions;
use crate::server::endpoints::signatures_for_address;
use crate::server::endpoints::holders;
use crate::server::endpoints::whales;
use crate::server::endpoints::new_spls;


pub async fn run_server() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(web::scope("/api")
                         .configure(signatures_for_address::init_routes)
                         .configure(holders::init_routes)
                         .configure(whales::init_routes)
                         .configure(transactions::init_routes)
                         .configure(accounts::init_routes)
            )
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
