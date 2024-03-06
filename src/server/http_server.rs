use actix_web::{App, HttpServer, web};
use std::sync::Arc;

use crate::server::endpoints::holders;
use crate::server::endpoints::whales;
use crate::server::endpoints::token_accounts_by_owner;


pub async fn run_server() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(web::scope("/api")
                         .configure(holders::init_routes)
                         .configure(whales::init_routes)
                         // .configure(analytics::init_routes) // Example for future modules
            )
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
