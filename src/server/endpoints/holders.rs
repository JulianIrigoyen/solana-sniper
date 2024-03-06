use std::collections::HashMap;
use std::error::Error;
use actix_web::{web, HttpResponse, Responder};
use reqwest::{Client, header};
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use solana_sdk::bs58;


pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/holders")
            .route(web::post().to(find_holders))
    );
}

async fn find_holders(request: web::Json<FindHoldersRequest>) -> impl Responder {
    let holders_data = process_mint_addresses(request.token_mint_addresses.clone()).await;
    match holders_data {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn process_mint_addresses(mint_addresses: Vec<String>) -> Result<HashMap<String, usize>, Box<dyn Error>> {
    println!("Finding holders for {:#?}", mint_addresses);
    let mut holders_count = HashMap::new();
    let client = Client::new();

    for mint_address in mint_addresses {
        let mint_address_base58 = bs58::encode(&mint_address).into_string();
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

        let response = client
            .post("https://necessary-light-mound.solana-mainnet.quiknode.pro/4c456991f88d116bec4eee1e69e4d0495b08ca42/")
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

                    let holder_count = response_json.result.iter()
                        .filter(|account| {
                            account.account.data.parsed.info.token_amount
                                .as_ref()
                                .map_or(false, |amount| amount.ui_amount > 0.0)
                        })
                        .count();

                    holders_count.insert(mint_address.clone(), holder_count.clone());

                    // Calculate the ratio of holder accounts to initialized accounts
                    let holder_ratio = if initialized_count > 0 {
                        holder_count.clone() as f64 / initialized_count as f64
                    } else {
                        0.0 // Avoid division by zero
                    };

                    println!("Initialized Accounts: {}", initialized_count);
                    println!("Holder Accounts: {}", holder_count.clone());
                    println!("Holder Ratio: {:.2}", holder_ratio);

                    // You might want to adjust what you insert into holders_count based on your requirements
                    holders_count.insert(mint_address.clone(), holder_count);
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

    Ok(holders_count)
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