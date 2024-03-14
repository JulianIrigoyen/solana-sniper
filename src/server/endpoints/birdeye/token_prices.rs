use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use actix_web::{HttpResponse, Responder, web};

// https://docs.birdeye.so/reference/post_defi-multi-price
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MultiPriceRequest {
    pub(crate) list_address: String, // comma separated addresses
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MultiPriceResponse {
    data: HashMap<String, TokenData>, //signature - data
    success: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenDataWithId {
    pub(crate) program_id: String,
    pub value: f64,
    #[serde(rename = "updateUnixTime")]
    pub update_unix_time: f64,
    #[serde(rename = "updateHumanTime")]
    pub update_human_time: String,
    #[serde(rename = "priceChange24h")]
    pub price_change_24_h: f64,
    pub liquidity: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenData {
    value: f64,
    #[serde(rename = "updateUnixTime")]
    update_unix_time: f64,
    #[serde(rename = "updateHumanTime")]
    update_human_time: String,
    #[serde(rename = "priceChange24h")]
    price_change_24_h: f64,
    liquidity: f64,
}

pub fn init_routes(cfg: &mut web::ServiceConfig){
    cfg.service(web::resource("/token-prices")
        .route(web::post().to(find_token_prices))
    );
}

pub async fn find_token_prices(request: web::Json<MultiPriceRequest>) -> impl Responder {
    let price_data = fetch_multi_token_prices(request).await;
    match price_data {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => {
            eprintln!("Error while fetching prices: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}



pub async fn fetch_multi_token_prices(request: web::Json<MultiPriceRequest>) -> Result<Vec<TokenDataWithId>, Box<dyn Error>> {
    let mut token_data: Vec<TokenDataWithId> = Vec::new();

    let client = reqwest::Client::new();

    //always bring liquidity by default
    let api_url = "https://public-api.birdeye.so/defi/multi_price?include_liquidity=true";

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    let birdeye_api_key = env::var("BIRDEYE_API_KEY").expect("BIRDEYE_API_KEY must be set");
    headers.insert("X-API-KEY", HeaderValue::from_str(&birdeye_api_key)?);


    let response = client
        .post(api_url)
        .headers(headers)
        .json(&request)
        .send()
        .await?;


    if response.status().is_success() {
        let response_text = response.text().await?;
        let value: serde_json::Value = serde_json::from_str(&response_text)?;
        println!("Got token data: {:#?}", value);

        match serde_json::from_value::<MultiPriceResponse>(value) {
            Ok(multi_price_response) => {
                for (id, data) in multi_price_response.data {
                    token_data.push(TokenDataWithId {
                        program_id: id,
                        value: data.value,
                        update_unix_time: data.update_unix_time,
                        update_human_time: data.update_human_time,
                        price_change_24_h: data.price_change_24_h,
                        liquidity: data.liquidity,
                    })
                }
            }
            Err(e) => {
                eprintln!("[[TOKEN PRICE]] Failed to deserialize response: {:?}", e);
            }
        }

    }

    Ok(token_data)
}
