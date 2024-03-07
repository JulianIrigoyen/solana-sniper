use actix_web::web::block;
use actix_web::{web, HttpResponse, Responder};
use reqwest::{Client, header};
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use solana_sdk::bs58;
use rust_decimal::{prelude::FromPrimitive, Decimal};

use lazy_static::lazy_static;
use hashbrown::HashMap;
use std::os::macos::fs;
use std::sync::Mutex;

use std::{env, thread};

use chrono::Local;
use csv::Writer;
use std::fs::File;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::error::Error;


//https://solana.com/es/docs/rpc/http/gettokenlargestaccounts
type TokenSupplyMap = HashMap<(String, u8), TokenSupply>;


lazy_static! {
    static ref TOKEN_SUPPLIES: Mutex<TokenSupplyMap> = Mutex::new({
        let mut m = TokenSupplyMap::new();
        m.insert(
            ("RETARDIO".to_string(), 6), 
            TokenSupply {
                amount: "999741137074833".to_string(),
                decimals: 6,
                ui_amount: 999741137.074833,
                ui_amount_string: "999741137.074833".to_string(),
            }
        );
        m
    });
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct FindWhalesRequest {
    pub token_mint_addresses: Vec<String>,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct TokenSupplyRpcResponse {
    pub jsonrpc: String,
    pub result: TokenSupplyRpcResult,
    pub id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenSupplyRpcResult {
    pub context: RpcContext,
    pub value: TokenSupply,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RpcContext {
    pub slot: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenSupply {
    pub amount: String,
    pub decimals: u8,
    #[serde(rename = "uiAmount")]
    pub ui_amount: f64,
    #[serde(rename = "uiAmountString")]
    pub ui_amount_string: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WhaleAccountsRpcResponse {
    pub jsonrpc: String,
    pub result: WhaleAccountRpcResult,
    pub id: u64,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct WhaleAccountRpcResult {
    pub context: RpcContext,
    pub value: Vec<WhaleAccount>,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
struct WhaleAccount {
    address: String,
    amount: String,
    decimals: u8,
    #[serde(rename = "uiAmount")]
    ui_amount: f64, 
    #[serde(rename = "uiAmountString")]
    ui_amount_string: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct WhaleDetail {
    pub address: String,
    pub amount: String,
    pub decimals: u8,
    pub ui_amount_string: String,
    pub owned_percentage: Decimal
}


pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/whales")
            .route(web::post().to(find_whales))
    );
}

async fn find_whales(request: web::Json<FindWhalesRequest>) -> impl Responder {
    let whale_data = get_largest_accounts_for_mints(request.token_mint_addresses.clone()).await;
    match whale_data {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn get_largest_accounts_for_mints(mint_addresses: Vec<String>) -> Result<Vec<WhaleDetail>, Box<dyn Error>> {
    println!("Finding whales for {:#?}", mint_addresses);
    let solana = env::var("PRIVATE_SOLANA_QUICKNODE").expect("PRIVATE_SOLANA_QUICKNODE must be set");
    let client = Client::new();
    let mut all_whales: Vec<WhaleDetail> = Vec::new();

    for mint_address in &mint_addresses {
        // Fetch the total supply for the mint address
        if let Ok(supply) = get_token_supply(&client, &mint_address).await {
            let total_supply: Decimal = Decimal::from_f64(supply.ui_amount).unwrap_or_else(|| Decimal::new(0, 0));

            println!("Total supply for {:#?} is {:#?}", mint_address.clone(), total_supply.clone());

            let rpc_request = json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "getTokenLargestAccounts",
                "params": [mint_address]
            });

            println!("Requesting whales  for {:#?} ::: {:#?}", mint_address, rpc_request);

            let response = client
                .post(solana.clone())
                .header("Content-Type", "application/json")
                .json(&rpc_request)
                .send()
                .await?;

            if response.status().is_success() {
                let response_text = response.text().await?;
                let value: serde_json::Value = serde_json::from_str(&response_text)?;
                println!("Largest account holders for {:#?} are: {:#?}", mint_address, value);
                let largest_accounts: WhaleAccountsRpcResponse = serde_json::from_str(&response_text)?;

                for account in largest_accounts.result.value {
                    let ui_amount: Decimal = account.ui_amount_string.parse::<Decimal>()?;

                    let owned_percentage = (ui_amount / total_supply) * Decimal::from(100);

                    println!("WHALE {:#?} own {:#?} % of {:#?}", account.address.clone(), owned_percentage, mint_address );


                    all_whales.push(WhaleDetail {
                        address: account.address,
                        amount: account.amount,
                        decimals: account.decimals,
                        ui_amount_string: account.ui_amount_string,
                        owned_percentage, // Include the ownership percentage
                    });
                }
            } else {
                println!("Error fetching data for mint address: {}", mint_address);
            }
        }
    }

    println!("ALL THE WHALES FOR {:#?}: {:#?}", &mint_addresses, all_whales);

    if let Some(a) = mint_addresses.get(0) {
        let result = write_whales_to_csv(a, &all_whales).await;
        if let Err(error) = result {
            println!("Failed to write whale details to CSV: {:#?}", error);
        }
    } else {
        println!("No mint addresses found.");
    }
    Ok(all_whales)
}


pub async fn get_token_supply(client: &Client, mint_address: &str) -> Result<TokenSupply, Box<dyn Error>> {
    println!("Finding token supply for {:#?}", mint_address);

    let rpc_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getTokenSupply",
        "params": [mint_address]
    });

    let response = client
        .post(env::var("PRIVATE_SOLANA_QUICKNODE").expect("PRIVATE_SOLANA_QUICKNODE must be set"))
        .header("Content-Type", "application/json")
        .json(&rpc_request)
        .send()
        .await?;

    match response.status() {
        reqwest::StatusCode::OK => {
            let response_text = response.text().await?;
            println!("Token supply for {:#?}: {:#?}", mint_address, response_text);
            let supply_response: TokenSupplyRpcResponse = serde_json::from_str(&response_text)?;
            println!("Deserialized token response ::: {:#?}", supply_response);
            Ok(supply_response.result.value)
        },
        _ => {
            let error_message = format!("Error fetching token supply for mint address: {}", mint_address);
            eprintln!("{}", error_message);
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, error_message)))
        },
    }
}

pub async fn write_whales_to_csv(program_address: &str, whales: &[WhaleDetail]) -> Result<(), Box<dyn Error>> {
    let date = Local::now().format("%Y-%m-%d").to_string();
    let dir_path = PathBuf::from(format!("data/{}/{}/whales", program_address, date));
    
    // Ensure the directory exists
    std::fs::create_dir_all(&dir_path)?;

    let file_path = dir_path.join("whale_details.csv");
    // Handle the Result returned by File::create using `?` to propagate errors
    let file = File::create(&file_path)?;
    let mut wtr = Writer::from_writer(file);

    for whale in whales {
        // Here, `serialize` writes each whale detail to the CSV.
        // If an error occurs, it will be converted to Box<dyn Error> and returned
        wtr.serialize(whale)?;
    }

    // Explicitly handle flush to ensure data is written
    wtr.flush()?;
    Ok(())
}