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
use crate::server::endpoints::holders;

use crate::server::endpoints::whales::get_token_supply;


pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/holders")
            .route(web::post().to(find_holders))
    );
}

#[derive(Serialize, Deserialize, Debug)]
struct HolderStats {
    mint_address: String,
    token_supply: Option<Decimal>,
    initialized_accounts: usize,
    holder_accounts: usize,
    holder_ratio: f64,
    categories: Option<HolderCategories>,
}

#[derive(Serialize, Deserialize, Debug)]
struct HolderCategories {
    micro: usize,
    small: usize,
    medium: usize,
    large: usize,
    major: usize,
    whale: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct CategoryDetail {
    holders: usize,
    max_supply_percentage: f64,
    token_amount_range: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct HolderDetailedStats {
    mint_address: String,
    token_supply: Option<Decimal>,
    initialized_accounts: usize,
    holder_accounts: usize,
    holder_ratio: f64,
    categories: HashMap<String, CategoryDetail>,
}
async fn find_holders(request: web::Json<FindHoldersRequest>) -> impl Responder {
    let holder_stats = process_mint_addresses(request.token_mint_addresses.clone()).await;
    match holder_stats {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn process_mint_addresses(mint_addresses: Vec<String>) -> Result<Vec<HolderDetailedStats>, Box<dyn Error>> {
    println!("Finding holders for {:#?}", mint_addresses);
    let mut results: Vec<HolderDetailedStats> = Vec::new();
    let client = Client::new();

    for mint_address in mint_addresses {
        let mint_address_base58 = bs58::encode(&mint_address).into_string();

        let token_supply_result = get_token_supply(&client, mint_address.as_str()).await;
        let supply: Option<Decimal> = if let Ok(token_supply) = token_supply_result {
            Some(Decimal::from_f64(token_supply.ui_amount).unwrap_or_else(|| Decimal::zero()))
        } else {
            None
        };
        println!("GOT TOKEN SUPPLY ::: {:#?}", supply);


        let params = json!([
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            {
                "encoding": "jsonParsed",
                    "filters": [
                        {
                            "dataSize": 165 // Expected size of a SPL Token account
                        },
                        {
                            "memcmp": {
                                "offset": 0, // Offset for the mint address in the account data
                                "bytes": mint_address // The mint address you're interested in
                            }
                        }
                    ]
            }
        ]);

        let rpc_request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getProgramAccounts",
            "params": params
        });

        // Construct headers
        let mut headers = header::HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
        let rpc_url = env::var("PRIVATE_SOLANA_QUICKNODE").expect("PRIVATE_SOLANA_QUICKNODE must be set");
        let response = client
            .post(rpc_url)
            .headers(headers)
            .json(&rpc_request)
            .send()
            .await?;

        if response.status().is_success() {
            let response_text = response.text().await?;

            // TODO most useful print ever
            //let value: serde_json::Value = serde_json::from_str(&response_text)?;
            // println!("{:#?}", value);

            match serde_json::from_str::<SolanaRpcResponse>(&response_text) {
                Ok(response_json) => {
                    let initialized_count = response_json.result.len();
                    let mut holder_category_count = HolderCategories {
                        micro: 0,
                        small: 0,
                        medium: 0,
                        large: 0,
                        major: 0,
                        whale: 0,
                    };


                    let mut non_empty_wallet_count = 0; // Track wallets with more than 0 tokens

                    response_json.result.iter().for_each(|account| {
                        let ui_amount = account.account.data.parsed.info.token_amount
                            .as_ref()
                            .map_or(0.0, |token_amount| token_amount.ui_amount);

                        if ui_amount > 0.0 {
                            non_empty_wallet_count += 1; // Only increment for non-empty wallets
                        }

                        let ui_amount_decimal = Decimal::from_f64(ui_amount).unwrap_or(Decimal::zero());

                        let percentage_of_total_supply = supply
                            .map(|s| if !s.is_zero() { (ui_amount_decimal / s) * Decimal::from(100) } else { Decimal::zero() })
                            .unwrap_or(Decimal::zero())
                            .round_dp(4); // Ensure rounding for clarity

                        // Category assignment based on the percentage of total supply
                        match percentage_of_total_supply {
                            _ if percentage_of_total_supply > Decimal::from_f64(1.0).unwrap() => holder_category_count.whale += 1,
                            _ if percentage_of_total_supply > Decimal::from_f64(0.1).unwrap() => holder_category_count.major += 1,
                            _ if percentage_of_total_supply > Decimal::from_f64(0.05).unwrap() => holder_category_count.large += 1,
                            _ if percentage_of_total_supply > Decimal::from_f64(0.01).unwrap() => holder_category_count.medium += 1,
                            _ if percentage_of_total_supply > Decimal::from_f64(0.001).unwrap() => holder_category_count.small += 1,
                            _ if percentage_of_total_supply <= Decimal::from_f64(0.0001).unwrap() => holder_category_count.micro += 1,
                            _ => (),
                        };
                    });

                    //todo market cap: token price * circulating supply

                    let mut category_detail: HashMap<String, CategoryDetail>;

                    if let Some(supply) = supply {
                        category_detail = HashMap::from([
                            ("micro".to_string(), CategoryDetail {
                                holders: holder_category_count.micro,
                                max_supply_percentage: 0.0001,
                                token_amount_range: format!("{:.0} - {:.0} tokens",
                                                            Decimal::zero(),
                                                            supply * Decimal::from_f64(0.000001).unwrap()),
                            }),
                            ("small".to_string(), CategoryDetail {
                                holders: holder_category_count.small,
                                max_supply_percentage: 0.001,
                                token_amount_range: format!("{:.0} - {:.0} tokens",
                                                            supply * Decimal::from_f64(0.00001).unwrap() + Decimal::one(),
                                                            supply * Decimal::from_f64(0.0001).unwrap()),
                            }),
                            ("medium".to_string(), CategoryDetail {
                                holders: holder_category_count.medium,
                                max_supply_percentage: 0.01,
                                token_amount_range: format!("{:.0} - {:.0} tokens",
                                                            supply * Decimal::from_f64(0.0001).unwrap(),
                                                            supply * Decimal::from_f64(0.001).unwrap() - Decimal::one()),
                            }),
                            ("large".to_string(), CategoryDetail {
                                holders: holder_category_count.large,
                                max_supply_percentage: 0.05,
                                token_amount_range: format!("{:.0} - {:.0} tokens",
                                                            supply * Decimal::from_f64(0.001).unwrap(),
                                                            supply * Decimal::from_f64(0.01).unwrap() - Decimal::one()),
                            }),
                            ("major".to_string(), CategoryDetail {
                                holders: holder_category_count.major,
                                max_supply_percentage: 0.1,
                                token_amount_range: format!("{:.0} - {:.0} tokens",
                                                            supply * Decimal::from_f64(0.01).unwrap(),
                                                            supply * Decimal::from_f64(0.1).unwrap() - Decimal::one()),
                            }),
                            ("whale".to_string(), CategoryDetail {
                                holders: holder_category_count.whale,
                                max_supply_percentage: 1.00,
                                token_amount_range: format!(">{:.0} tokens",
                                                            supply * Decimal::from_f64(0.1).unwrap()),
                            }),
                        ]);
                    } else {
                        category_detail = HashMap::new()
                    }


                    // Now, calculate holder_ratio based on non-empty wallets
                    let holder_ratio = if initialized_count.clone() > 0 {
                        non_empty_wallet_count.clone() as f64 / initialized_count.clone() as f64
                    } else {
                        0.0 // Avoid division by zero
                    };

                    //todo -> store non_empty_wallet_count
                    //todo -> reshoot this call after n minutes
                    //todo -> fetch previous non_empty_wallet_count
                    //todo -> calculate delta = new_non_empty_wallet_count -  previous_nonempty_wallet_count
                    //todo -> calculate calcaute % and store
                    //todo -> alert

                    let stats = HolderDetailedStats {
                        mint_address: mint_address.clone(),
                        token_supply: Some(supply.unwrap_or(Decimal::zero())),
                        initialized_accounts: initialized_count,
                        holder_accounts: non_empty_wallet_count,
                        holder_ratio: holder_ratio,
                        categories: category_detail,
                    };
                    println!("{:#?}", stats);
                    results.push(stats);
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
    }

    Ok(results)
}

async fn pretty_print_response(response: SolanaRpcResponse) -> Result<(), Box<dyn std::error::Error>> {
    // Iterate through the accounts and print the details
    println!("Parsed Response:");
    for account in response.result.clone() {
        println!("Public Key: {}", &account.pubkey);
        println!("  Owner: {}", &account.account.data.parsed.info.owner);
        println!("  Mint: {}", &account.account.data.parsed.info.mint);
        // println!("  Token Amount: {}", account.account.data.parsed.info.token_amount
        //     .as_ref() // Convert Option<TokenAmount> to Option<&TokenAmount>
        //     .map_or("0".to_string(), |token_amount| token_amount.ui_amount_string.clone()));

        println!("  State: {}", account.account.data.parsed.info.state);
        println!("-------------------------------------------------");
    }

    Ok(())
}


#[derive(Serialize, Deserialize, Clone, Debug)]
struct FindHoldersRequest {
    pub token_mint_addresses: Vec<String>,
}

/// Requet Structure
#[derive(Serialize, Deserialize, Clone, Debug)]
struct SolanaRpcFilter {
    pub memcmp: Option<SolanaRpcMemcmp>,
    pub dataSize: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct SolanaRpcMemcmp {
    pub offset: u64,
    pub bytes: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct SolanaRpcRequestParams {
    pub filters: Vec<SolanaRpcFilter>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct SolanaRpcRequest {
    pub jsonrpc: String,
    pub id: u8,
    pub method: String,
    pub params: Vec<serde_json::Value>, // The second parameter can have a complex structure
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SolanaRpcResponse {
    jsonrpc: String,
    result: Vec<AccountInfo>,
    id: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AccountInfo {
    pubkey: String,
    account: AccountData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AccountData {
    data: AccountDataDetails,
    executable: bool,
    lamports: u64,
    owner: String,
    rentEpoch: u64,
    space: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct AccountDataDetails {
    parsed: ParsedAccountData,
    program: String,
    space: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ParsedAccountData {
    info: TokenAccountInfo,
    #[serde(rename = "type")]
    account_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TokenAccountInfo {
    #[serde(rename = "isNative")]
    is_native: bool,
    mint: String,
    owner: String,
    state: String,
    #[serde(rename = "tokenAmount")]
    token_amount: Option<TokenAmount>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TokenAmount {
    amount: String,
    decimals: u8,
    #[serde(rename = "uiAmount")]
    ui_amount: f64,
    #[serde(rename = "uiAmountString")]
    ui_amount_string: String,
}