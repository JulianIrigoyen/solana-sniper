use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use reqwest::{Client, header};
use serde_json::{json, Value};
use solana_sdk::bs58;


#[derive(Serialize, Deserialize, Clone, Debug)]
struct SignaturesResponse {
    pub signatures: Vec<String>,
    pub count: usize
}

#[derive(Serialize, Deserialize, Debug)]
struct GetSignaturesResponse {
    jsonrpc: String,
    result: Vec<SignatureResult>,
    id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct SignatureResult {
    err: Option<Value>, // Use more specific type if error structure is known
    memo: Option<String>,
    signature: String,
    slot: u64,
    blockTime: Option<i64>,
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/signatures")
            .route(web::post().to(find_signatures_for_address))
    );
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct FindSignaturesForAddressRequest {
    pub address: String,
}

async fn find_signatures_for_address(request: web::Json<FindSignaturesForAddressRequest>) -> impl Responder {
    let address = &request.address;
    let signature_data = get_signatures(address).await;
    match signature_data {
        Ok((signatures, count)) => HttpResponse::Ok().json(SignaturesResponse { signatures, count }),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}


async fn get_signatures(address: &String) -> Result<(Vec<String>, usize), Box<dyn Error>> {
    println!("Getting signatures for {:#?}", address);
    let mut signatures = Vec::new();
    let client = Client::new();
    let solana_rpc_url = env::var("PRIVATE_SOLANA_QUICKNODE").expect("PRIVATE_SOLANA_QUICKNODE must be set");
    let current_timestamp = Utc::now().timestamp_millis();

    let mut headers = header::HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());

    let rpc_request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getSignaturesForAddress",
            "params": [
              address,
              {
                "limit": 1000,
                 "before": "3uLQNx9U85TnmaBWLrjQHEqauADbjhQEQriVtVY2yDzHWYNwZ1BBMMtZcjTQvkkT7bV7nT7z9B8h3zJ3xz9RYU5w"
              }
            ]
        });

    let response= client
        .post(&solana_rpc_url)
        .headers(headers)
        .json(&rpc_request)
        .send()
        .await?;

    if response.status().is_success(){
        let response_text = response.text().await?;
        let value: serde_json::Value = serde_json::from_str(&response_text)?;
        println!("Got signatures for address {:#?}: {:#?}", address, value);
        match serde_json::from_str::<GetSignaturesResponse>(&response_text) {
            Ok(response_json) => {
                   for sig in response_json.result {
                       signatures.push(sig.signature);
                   }

            }
            Err(e) => {
                eprintln!("Failed to parse response JSON: {:?}", e);
            }
        }
    } else {
        // If the response status is not successful, log the error response body
        let error_text = response.text().await?;
        eprintln!("Error fetching data for mint address: {}", error_text);
    }
    let count = signatures.len();
    let result = (signatures, count);
    Ok(result)
}