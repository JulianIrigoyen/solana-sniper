use reqwest::{Client as ReqwestClient, Response};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use crate::http::http_client_error::HttpClientError;

pub struct BaseHttpClient {
    client: ReqwestClient,
    default_headers: HashMap<String, String>,
}

impl BaseHttpClient {
    pub fn new() -> Self {
        Self {
            client: ReqwestClient::new(),
            default_headers: HashMap::new(),
        }
    }

    pub fn add_default_header(mut self, key: &str, value: &str) -> Self {
        self.default_headers.insert(key.to_string(), value.to_string());
        self
    }

    async fn send_request<T: DeserializeOwned>(&self, method: reqwest::Method, url: &str, headers: Option<HashMap<&str, &str>>, body: Option<String>) -> Result<T, Box<dyn std::error::Error>> {
        let mut request = self.client.request(method, url);

        // Apply default headers
        for (key, value) in &self.default_headers {
            request = request.header(key, value);
        }

        // Apply additional headers
        if let Some(hdrs) = headers {
            for (key, value) in hdrs {
                request = request.header(key, value);
            }
        }

        // Set JSON body if present
        if let Some(b) = body {
            request = request.body(b);
        }

        let response: Response = request.send().await?;
        let status = response.status();
        let result = response.json::<T>().await?;

        if status.is_success() {
            Ok(result)
        } else {
            Err(format!("Request failed with status: {}", status).into())
        }
    }

    pub async fn get<T: DeserializeOwned>(&self, url: &str, headers: Option<HashMap<&str, &str>>) -> Result<T, Box<dyn std::error::Error>> {
        self.send_request(reqwest::Method::GET, url, headers, None).await
    }

    pub async fn post<T: DeserializeOwned>(&self, url: &str, headers: Option<HashMap<&str, &str>>, body: String) -> Result<T, Box<dyn std::error::Error>> {
        self.send_request(reqwest::Method::POST, url, headers, Some(body)).await
    }

    pub async fn get_with_query<T: DeserializeOwned>(&self, url: &str, query: &[(&str, &str)]) -> Result<T, HttpClientError> {
        let request = self.client.get(url);

        // Apply default headers
        let mut request = request;
        for (key, value) in &self.default_headers {
            request = request.header(key, value);
        }

        // Apply query parameters
        let request = request.query(query);

        // Send the request
        let response = request.send().await.map_err(HttpClientError::from)?;

        // Check for HTTP success status
        if !response.status().is_success() {
            let error = match response.status() {
                reqwest::StatusCode::UNAUTHORIZED => HttpClientError::Unauthorized,
                reqwest::StatusCode::BAD_REQUEST => HttpClientError::BadRequest("Bad request".into()),
                reqwest::StatusCode::NOT_FOUND => HttpClientError::NotFound,
                _ => HttpClientError::Other(response.error_for_status().unwrap_err()), // Convert to reqwest::Error for detailed error
            };
            return Err(error);
        }

        // Deserialize the response body into the expected type
        response.json::<T>().await.map_err(HttpClientError::from)
    }
}
