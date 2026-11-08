use thiserror::Error;

mod jsonrpc_error {
    pub const INTERNAL_ERROR: i32 = -32603;
    pub const INVALID_PARAMS: i32 = -32602;
}

#[derive(Error, Debug)]
pub enum WebsurfxError {
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

    #[error("Websurfx API returned an error: {0}")]
    ApiError(String),

    #[error("Websurfx API returned HTTP {status}: {body}")]
    HttpStatusError { status: u16, body: String },

    #[error("No results found for the given query")]
    NoResults,

    #[error("URL parsing error: {0}")]
    UrlError(#[from] url::ParseError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("MCP server error: {0}")]
    McpError(String),
}
pub type WebsurfxResult<T> = Result<T, WebsurfxError>;

impl WebsurfxError {
    pub fn user_message(&self) -> String {
        match self {
            WebsurfxError::InvalidBaseUrl(url) => {
                format!("Invalid Websurfx server URL: {}", url)
            }
            WebsurfxError::HttpError(e) => {
                format!("Failed to connect to Websurfx server: {}", e)
            }
            WebsurfxError::JsonParseError(_) => {
                "Received invalid response from Websurfx server".to_string()
            }
            WebsurfxError::InvalidParameters(msg) => {
                format!("Invalid search parameters: {}", msg)
            }
            WebsurfxError::InvalidEnvVar(msg) => {
                format!("Invalid environment variable: {}", msg)
            }
            WebsurfxError::ApiError(msg) => {
                format!("Websurfx API error: {}", msg)
            }
            WebsurfxError::HttpStatusError { status, body } => {
                format!("Websurfx API error (HTTP {}): {}", status, body)
            }
            WebsurfxError::NoResults => "No search results found for the given query".to_string(),
            WebsurfxError::UrlError(_) => "Invalid URL format".to_string(),
            WebsurfxError::IoError(e) => {
                format!("IO error: {}", e)
            }
            WebsurfxError::McpError(msg) => {
                format!("MCP server error: {}", msg)
            }
        }
    }
}

impl From<WebsurfxError> for rmcp::ErrorData {
    fn from(err: WebsurfxError) -> Self {
        let error_code = match err {
            WebsurfxError::InvalidParameters(_) => jsonrpc_error::INVALID_PARAMS,
            _ => jsonrpc_error::INTERNAL_ERROR,
        };

        rmcp::ErrorData::new(rmcp::model::ErrorCode(error_code), err.user_message(), None)
    }
}
