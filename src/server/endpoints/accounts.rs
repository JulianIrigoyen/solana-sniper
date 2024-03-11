use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::str::FromStr;
use actix_web::{web, HttpResponse, Responder};
use diesel::serialize::IsNull::No;
use log::Level::Debug;
use reqwest::{Client, header};
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use solana_sdk::bs58;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use rust_decimal::prelude::{One, Zero};
use solana_sdk::pubkey::Pubkey;
use crate::server::endpoints::holders;

use crate::server::endpoints::whales::get_token_supply;

pub fn init_routes(cfg: &mut web::ServiceConfig){
    cfg.service(web::resource("/accounts")
        .route(web::post().to(find_accounts))
    );
}

#[derive(Serialize, Deserialize, Debug)]
struct FindAccountsRequest {
    account_addresses: Vec<String>
}

pub async fn find_accounts(request: web::Json<FindAccountsRequest>) -> impl Responder {
    let account_data = process_account_addresses(request.account_addresses.clone()).await;
    match account_data {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn process_account_addresses(addresses: Vec<String>) -> Result<Vec<AccountDetail>, Box<dyn std::error::Error>> {
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let mut accounts: Vec<AccountDetail> = Vec::new();
    let client = Client::new();

    println!("Fetching accounts:  {:#?}", addresses);

    for address in addresses {
        let params = json!([
            address,
            {
                "encoding": "base64"
            }
        ]);
        let rpc_request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getAccountInfo",
            "params": params
        });

        let response = client.post(rpc_url)
            .header("Content-Type", "application/json")
            .json(&rpc_request)
            .send()
            .await?;

        if response.status().is_success() {
            let response_text = response.text().await?;
            let value: serde_json::Value = serde_json::from_str(&response_text)?;
            println!("Got Account: {:#?}", value);

            if let Ok(account_info) = serde_json::from_str::<AccountInfoResponse>(&response_text) {
                if let Some(result) = account_info.result {
                    accounts.push(AccountDetail {
                        address: address.clone(),
                        lamports: result.value.lamports,
                        owner: result.value.owner,
                        executable: result.value.executable,
                        rent_epoch: result.value.rent_epoch,
                    });
                }
            }
        }
    }

    Ok(accounts)
}


///To get the IDL (Interface Definition Language) for a  program,
/// involves a bit more specific steps than just fetching account information because the IDL is stored in a specific account associated with the program. The IDL account is a particular account that Anchor uses to store the program's IDL, making it accessible for clients to understand how to interact with the program.
pub async fn process_idl(idl_address: String) -> Result<Vec<AccountDetail>, Box<dyn std::error::Error>> {
    // Calculate the PDA for the IDL account.
    println!("Fetching IDL for program {:#?}", idl_address);
    let program_pubkey = Pubkey::from_str(&*idl_address)?;
    let seeds = &[b"anchor:idl".as_ref(), program_pubkey.as_ref()];
    let (idl_address, _) = Pubkey::find_program_address(seeds, &program_pubkey);

    // The IDL data is compressed using zstd. First, skip the 8-byte discriminator.
    let account_data = process_account_addresses(vec![idl_address.to_string()]).await?;
    let idl_data = &account_data[8..];

    Ok(vec![AccountDetail { address: "val".to_string(), lamports: 1, owner: "val".to_string(), executable: true, rent_epoch: 1 }])
}

/**
Returns transaction details for a confirmed transaction. Params:
 - Transaction address vector, as base-58 encoded strings
 */
#[derive(Serialize, Debug)]
struct AccountDetail {
    address: String,
    lamports: u64,
    owner: String,
    executable: bool,
    rent_epoch: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct AccountInfoResponse {
    jsonrpc: String,
    result: Option<AccountInfoResult>,
    id: u8,
}

#[derive(Serialize, Deserialize, Debug)]
struct AccountInfoResult {
    value: AccountValue,
}

#[derive(Serialize, Deserialize, Debug)]
struct AccountValue {
    data: Vec<String>,
    executable: bool,
    lamports: u64,
    owner: String,
    rent_epoch: u64,
}




//
// async fn get_account_info(address: &str) -> Result<AccountInfoResponse, Box<dyn std::error::Error>> {
//     let rpc_url = "https://api.mainnet-beta.solana.com";
//     let request_headers = {
//         let mut headers = header::HeaderMap::new();
//         headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
//         headers
//     };
//
//     let client = Client::new();
//     let params = json!([
//         idl_address,
//         {
//             "encoding": "base64"
//         }
//     ]);
//     let rpc_request = json!({
//         "jsonrpc": "2.0",
//         "id": 1,
//         "method": "getAccountInfo",
//         "params": params
//     });
//
//     let response = client
//         .post(rpc_url)
//         .headers(request_headers)
//         .json(&rpc_request)
//         .send()
//         .await?;
//
//     if response.status().is_success() {
//         let response_text = response.text().await?;
//         // TODO most useful print ever
//         let value: serde_json::Value = serde_json::from_str(&response_text)?;
//         println!("{:#?}", value);
//         let account_info: AccountInfoResponse = serde_json::from_str(&response_text)?;
//         Ok(account_info)
//     } else {
//         Err(())
//     }
// }

