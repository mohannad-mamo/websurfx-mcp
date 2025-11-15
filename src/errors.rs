use thiserror::Error;

#[derive(Error, Debug)]
pub enum SearxngError {
    #[error("Environment variable SEARXNG_BASE_URL not set or invalid")]
    MissingBaseUrl,

    #[error("Invalid base URL: {0}")]
    InvalidBaseUrl(String),

    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Failed to parse JSON response: {0}")]
    JsonParseError(#[from] serde_json::Error),

    #[error("Invalid search parameters: {0}")]
    InvalidParameters(String),

    #[error("SearXNG API returned an error: {0}")]
    ApiError(String),

    #[error("No results found for the given query")]
    NoResults,

    #[error("URL parsing error: {0}")]
    UrlError(#[from] url::ParseError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("MCP server error: {0}")]
    McpError(String),
}
pub type SearxngResult<T> = Result<T, SearxngError>;

impl SearxngError {
    pub fn user_message(&self) -> String {
        match self {
            SearxngError::MissingBaseUrl => {
                "SearXNG server URL not configured. Please set SEARXNG_BASE_URL environment variable.".to_string()
            }
            SearxngError::InvalidBaseUrl(url) => {
                format!("Invalid SearXNG server URL: {}", url)
            }
            SearxngError::HttpError(e) => {
                format!("Failed to connect to SearXNG server: {}", e)
            }
            SearxngError::JsonParseError(_) => {
                "Received invalid response from SearXNG server".to_string()
            }
            SearxngError::InvalidParameters(msg) => {
                format!("Invalid search parameters: {}", msg)
            }
            SearxngError::ApiError(msg) => {
                format!("SearXNG API error: {}", msg)
            }
            SearxngError::NoResults => {
                "No search results found for the given query".to_string()
            }
            SearxngError::UrlError(_) => {
                "Invalid URL format".to_string()
            }
            SearxngError::IoError(e) => {
                format!("IO error: {}", e)
            }
            SearxngError::McpError(msg) => {
                format!("MCP server error: {}", msg)
            }
        }
    }
}

impl From<SearxngError> for rmcp::ErrorData {
    fn from(err: SearxngError) -> Self {
        rmcp::ErrorData::new(rmcp::model::ErrorCode(-32603), err.user_message(), None)
    }
}
