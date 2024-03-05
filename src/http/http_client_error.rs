use std::fmt;

#[derive(Debug)]
pub enum HttpClientError {
    Unauthorized,
    BadRequest(String),
    NotFound,
    Other(reqwest::Error),
}

impl fmt::Display for HttpClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            HttpClientError::Unauthorized => write!(f, "Unauthorized: Invalid API key or lack of permissions."),
            HttpClientError::BadRequest(ref message) => write!(f, "Bad Request: {}", message),
            HttpClientError::NotFound => write!(f, "Not Found: The requested resource could not be found."),
            HttpClientError::Other(ref e) => write!(f, "Other Error: {}", e),
        }
    }
}

impl std::error::Error for HttpClientError {}

impl From<reqwest::Error> for HttpClientError {
    fn from(error: reqwest::Error) -> Self {
        // Map specific reqwest errors to your custom errors
        if error.is_status() {
            match error.status() {
                Some(reqwest::StatusCode::UNAUTHORIZED) => HttpClientError::Unauthorized,
                Some(reqwest::StatusCode::BAD_REQUEST) => HttpClientError::BadRequest("Bad request".into()),
                Some(reqwest::StatusCode::NOT_FOUND) => HttpClientError::NotFound,
                _ => HttpClientError::Other(error),
            }
        } else {
            HttpClientError::Other(error)
        }
    }
}
