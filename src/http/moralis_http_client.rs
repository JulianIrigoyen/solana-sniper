use crate::http::base_http_client::BaseHttpClient;
use crate::http::http_client_error::HttpClientError;
use crate::models::moralis::token_price_response::ERC20TokenPriceResponse;


pub struct MoralisHttpClient {
    base_client: BaseHttpClient,
    base_url: String,
}

impl MoralisHttpClient {
    pub fn new(api_key: &str) -> Self {
        let base_client = BaseHttpClient::new()
            .add_default_header("X-API-Key", api_key)
            .add_default_header("accept", "application/json");

        Self {
            base_client,
            base_url: "https://deep-index.moralis.io/api/v2".to_string(),
        }
    }

    pub async fn get_token_price(&self, address: &str, chain: &str, include_percent_change: bool) -> Result<ERC20TokenPriceResponse, HttpClientError> {
        // Construct the full endpoint URL
        let endpoint = format!("erc20/{}/price", address);
        let url = format!("{}/{}", &self.base_url, endpoint);

        let query = [
            ("chain", chain),
            ("include", if include_percent_change { "percent_change" } else { "" }),
        ];

        // Use the base HTTP client to send the request
        self.base_client.get_with_query::<ERC20TokenPriceResponse>(&url, &query).await
    }

    // Add other Moralis-specific methods here...
}
