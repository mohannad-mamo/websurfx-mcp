use thiserror::Error;

mod jsonrpc_error {
    pub const INTERNAL_ERROR: i32 = -32603;
    pub const INVALID_PARAMS: i32 = -32602;
}

#[derive(Error, Debug)]
pub enum SearxngError {
    #[error("Invalid base URL: {0}")]
    InvalidBaseUrl(String),

    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Failed to parse JSON response: {0}")]
    JsonParseError(#[from] serde_json::Error),

    #[error("Invalid search parameters: {0}")]
    InvalidParameters(String),

    #[error("Invalid environment variable: {0}")]
    InvalidEnvVar(String),

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
            SearxngError::InvalidEnvVar(msg) => {
                format!("Invalid environment variable: {}", msg)
            }
            SearxngError::ApiError(msg) => {
                format!("SearXNG API error: {}", msg)
            }
            SearxngError::NoResults => "No search results found for the given query".to_string(),
            SearxngError::UrlError(_) => "Invalid URL format".to_string(),
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
        let error_code = match err {
            SearxngError::InvalidParameters(_) => jsonrpc_error::INVALID_PARAMS,
            _ => jsonrpc_error::INTERNAL_ERROR,
        };

        rmcp::ErrorData::new(rmcp::model::ErrorCode(error_code), err.user_message(), None)
    }
}
