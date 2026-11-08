use crate::config::Config;
use crate::errors::{WebsurfxError, WebsurfxResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const USER_AGENT: &str = "websurfx-mcp/0.2";
const DEFAULT_FORMAT: &str = "true";
const VALID_TIME_RANGES: &[&str] = &["day", "month", "year"];
const MAX_SAFE_SEARCH_LEVEL: u8 = 2;
const MAX_CONTENT_LENGTH: usize = 200;
const MAX_HTTP_CLIENT_POOL_SIZE: usize = 3;
const HTTP_CLIENT_IDLE_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_HTTP_REQUEST_RETRY: u32 = 3;

/// Accepts either a JSON string or an array of strings, and always returns
/// a joined string. Needed because Websurfx returns `engines`/`categories`
/// as arrays, while SearXNG-style clients expect a single string.
fn string_or_vec<'de, D>(d: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum S {
        One(String),
        Many(Vec<String>),
    }
    Ok(match Option::<S>::deserialize(d)? {
        Some(S::One(s)) => s,
        Some(S::Many(v)) => v.join(", "),
        None => String::new(),
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchParams {
    pub q: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engines: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pageno: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_range: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safesearch: Option<u8>,
}

impl SearchParams {
    pub fn new(query: String) -> Self {
        Self {
            q: query,
            json: Some(DEFAULT_FORMAT.to_string()),
            categories: None,
            engines: None,
            language: None,
            pageno: None,
            time_range: None,
            safesearch: None,
        }
    }

    pub fn with_categories(mut self, categories: String) -> Self {
        self.categories = Some(categories);
        self
    }

    pub fn with_engines(mut self, engines: String) -> Self {
        self.engines = Some(engines);
        self
    }

    pub fn with_language(mut self, language: String) -> Self {
        self.language = Some(language);
        self
    }

    pub fn with_page(mut self, page: u32) -> Self {
        self.pageno = Some(page);
        self
    }

    pub fn with_time_range(mut self, time_range: String) -> Self {
        self.time_range = Some(time_range);
        self
    }

    pub fn with_safe_search(mut self, level: u8) -> Self {
        self.safesearch = Some(level);
        self
    }

    pub fn validate(&self) -> WebsurfxResult<()> {
        if self.q.trim().is_empty() {
            return Err(WebsurfxError::InvalidParameters(
                "Query cannot be empty".to_string(),
            ));
        }

        if let Some(ref time_range) = self.time_range {
            if !VALID_TIME_RANGES.contains(&time_range.as_str()) {
                return Err(WebsurfxError::InvalidParameters(
                    "Time range must be 'day', 'month', or 'year'".to_string(),
                ));
            }
        }

        if let Some(safesearch) = self.safesearch {
            if safesearch > MAX_SAFE_SEARCH_LEVEL {
                return Err(WebsurfxError::InvalidParameters(
                    "Safe search level must be 0, 1, or 2".to_string(),
                ));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    #[serde(default, alias = "description")]
    pub content: String,
    #[serde(default, alias = "engines", deserialize_with = "string_or_vec")]
    pub engine: String,
    #[serde(default, alias = "categories", deserialize_with = "string_or_vec")]
    pub category: String,
    #[serde(default, alias = "relevanceScore")]
    pub score: f64,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct SearchResponse {
    #[serde(default)]
    pub query: String,
    #[serde(default)]
    pub number_of_results: u64,
    #[serde(default)]
    pub results: Vec<SearchResult>,
    #[serde(default)]
    pub engines: Vec<EngineInfo>,
    #[serde(default)]
    pub suggestions: Vec<String>,
    #[serde(default)]
    pub search: SearchInfo,
}

#[derive(Debug, Default, Deserialize)]
#[allow(dead_code)]
pub struct EngineInfo {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub count: u32,
}

#[derive(Debug, Default, Deserialize)]
#[allow(dead_code)]
pub struct SearchInfo {
    #[serde(default)]
    pub total_results: Option<u64>,
    #[serde(default)]
    pub search_time: Option<f64>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WebsurfxClient {
    client: Client,
    config: Config,
}

impl WebsurfxClient {
    #[allow(dead_code)]
    pub fn new(config: Config) -> WebsurfxResult<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .user_agent(USER_AGENT)
            .pool_max_idle_per_host(MAX_HTTP_CLIENT_POOL_SIZE)
            .pool_idle_timeout(HTTP_CLIENT_IDLE_TIMEOUT)
            .build()
            .map_err(WebsurfxError::HttpError)?;

        Ok(Self { client, config })
    }

    #[allow(dead_code)]
    pub async fn search(&self, mut params: SearchParams) -> WebsurfxResult<SearchResponse> {
        params.validate()?;

        // Ensure format is always set to json for API responses
        params.json = Some(DEFAULT_FORMAT.to_string());

        let response = self
            .client
            .get(self.config.search_url())
            .query(&params)
            .send()
            .await
            .map_err(WebsurfxError::HttpError)?;

        if !response.status().is_success() {
            return Err(WebsurfxError::ApiError(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        let mut search_response: SearchResponse =
            response.json().await.map_err(WebsurfxError::HttpError)?;

        // Websurfx does not echo the query; fill it in from the request
        if search_response.query.is_empty() {
            search_response.query = params.q.clone();
        }

        Ok(search_response)
    }

    #[allow(dead_code)]
    pub async fn search_with_retry(&self, params: SearchParams) -> WebsurfxResult<SearchResponse> {
        self.search_with_retry_impl(params, MAX_HTTP_REQUEST_RETRY)
            .await
    }

    fn is_retryable_error(error: &WebsurfxError) -> bool {
        match error {
            WebsurfxError::HttpError(reqwest_error) => {
                // Retry on network timeouts, connection errors, and 5xx server errors
                reqwest_error.is_timeout()
                    || reqwest_error.is_connect()
                    || reqwest_error
                        .status()
                        .is_some_and(|status| status.is_server_error())
            }
            WebsurfxError::ApiError(_) => {
                // Retry on API errors that might be transient
                true
            }
            _ => false,
        }
    }

    #[allow(dead_code)]
    async fn search_with_retry_impl(
        &self,
        params: SearchParams,
        max_retries: u32,
    ) -> WebsurfxResult<SearchResponse> {
        let mut last_error = None;

        for attempt in 0..max_retries {
            match self.search(params.clone()).await {
                Ok(response) => return Ok(response),
                Err(e) if attempt < max_retries - 1 && Self::is_retryable_error(&e) => {
                    let delay_ms = 100 * (attempt + 1) as u64;
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                    last_error = Some(e);
                    continue;
                }
                Err(e) => return Err(e),
            }
        }

        Err(last_error
            .unwrap_or_else(|| WebsurfxError::ApiError("Max retries exceeded".to_string())))
    }

    pub fn format_results(&self, response: &SearchResponse) -> String {
        if response.results.is_empty() {
            return "No results found for your search query.".to_string();
        }

        let mut formatted = String::new();

        formatted.push_str(&format!("Search Results for: \"{}\"\n", response.query));
        formatted.push_str(&format!("Found {} results\n\n", response.results.len()));

        let results_to_show = response.results.iter().take(self.config.max_results);

        for (index, result) in results_to_show.enumerate() {
            formatted.push_str(&format!("{}. {}\n", index + 1, result.title));
            formatted.push_str(&format!("   URL: {}\n", result.url));

            if !result.content.is_empty() {
                // Truncate on a UTF-8 character boundary. Slicing by bytes
                // (&result.content[..MAX_CONTENT_LENGTH]) panics for multi-byte
                // scripts like Arabic, Hebrew, CJK, etc.
                let content = if result.content.chars().count() > MAX_CONTENT_LENGTH {
                    let mut truncated: String =
                        result.content.chars().take(MAX_CONTENT_LENGTH).collect();
                    truncated.push_str("...");
                    truncated
                } else {
                    result.content.clone()
                };
                formatted.push_str(&format!("   {}\n", content));
            }

            if !result.engine.is_empty() {
                formatted.push_str(&format!("   Source: {}\n", result.engine));
            }

            formatted.push('\n');
        }

        if !response.suggestions.is_empty() {
            formatted.push_str("Suggestions:\n");
            for suggestion in &response.suggestions {
                formatted.push_str(&format!("  - {}\n", suggestion));
            }
            formatted.push('\n');
        }

        if !response.engines.is_empty() {
            formatted.push_str("Search engines used: ");
            let engine_names: Vec<String> =
                response.engines.iter().map(|e| e.name.clone()).collect();
            formatted.push_str(&engine_names.join(", "));
            formatted.push('\n');
        }

        formatted
    }

    #[allow(dead_code)]
    pub async fn test_connection(&self) -> WebsurfxResult<()> {
        self.config.validate().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_params_builder() {
        let params = SearchParams::new("test query".to_string())
            .with_categories("general,news".to_string())
            .with_language("en".to_string())
            .with_page(2)
            .with_time_range("month".to_string())
            .with_safe_search(1);

        assert_eq!(params.q, "test query");
        assert_eq!(params.categories, Some("general,news".to_string()));
        assert_eq!(params.language, Some("en".to_string()));
        assert_eq!(params.pageno, Some(2));
        assert_eq!(params.time_range, Some("month".to_string()));
        assert_eq!(params.safesearch, Some(1));
    }

    #[test]
    fn test_search_params_validation() {
        let valid_params = SearchParams::new("test query".to_string())
            .with_safe_search(1)
            .with_time_range("day".to_string());
        assert!(valid_params.validate().is_ok());

        let invalid_safe_search = SearchParams::new("test query".to_string()).with_safe_search(5); // Invalid: > 2
        assert!(invalid_safe_search.validate().is_err());

        let invalid_time_range =
            SearchParams::new("test query".to_string()).with_time_range("invalid".to_string());
        assert!(invalid_time_range.validate().is_err());

        let empty_query = SearchParams::new("   ".to_string());
        assert!(empty_query.validate().is_err());
    }

    #[test]
    fn test_search_params_serialization() {
        let params = SearchParams::new("test query".to_string())
            .with_categories("general,news".to_string())
            .with_language("en".to_string())
            .with_page(2);

        let mut params_with_json = params.clone();
        params_with_json.json = Some("true".to_string());

        let serialized = serde_json::to_string(&params_with_json).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();

        assert!(parsed.get("q").is_some());
        assert!(parsed.get("json").is_some());
        assert!(parsed.get("categories").is_some());
        assert!(parsed.get("language").is_some());
        assert!(parsed.get("pageno").is_some());

        assert!(parsed.get("engines").is_none());
        assert!(parsed.get("time_range").is_none());
        assert!(parsed.get("safesearch").is_none());

        let minimal_params = SearchParams::new("minimal".to_string());
        let minimal_serialized = serde_json::to_string(&minimal_params).unwrap();
        let minimal_parsed: serde_json::Value = serde_json::from_str(&minimal_serialized).unwrap();

        assert_eq!(minimal_parsed.as_object().unwrap().len(), 2);
        assert!(minimal_parsed.get("q").is_some());
        assert!(minimal_parsed.get("json").is_some());
    }

    #[tokio::test]
    async fn test_client_creation() {
        let config = Config::default();
        let client = WebsurfxClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_retryable_error_detection() {
        let api_error = WebsurfxError::ApiError("server busy".to_string());
        assert!(WebsurfxClient::is_retryable_error(&api_error));

        let param_error = WebsurfxError::InvalidParameters("invalid query".to_string());
        assert!(!WebsurfxClient::is_retryable_error(&param_error));

        let no_results_error = WebsurfxError::NoResults;
        assert!(!WebsurfxClient::is_retryable_error(&no_results_error));
    }

    #[tokio::test]
    #[ignore = "requires a running Websurfx instance at localhost:8080"]
    async fn test_search_integration() {
        use std::env;

        env::set_var("WEBSURFX_BASE_URL", "http://localhost:8080");
        let config = Config::from_env().expect("Failed to load config");
        let client = WebsurfxClient::new(config).expect("Failed to create client");

        let params = SearchParams::new("rust programming language".to_string())
            .with_language("en".to_string());

        let result = client.search(params).await;

        match result {
            Ok(response) => {
                assert!(!response.query.is_empty());
                println!("Search successful: {} results", response.results.len());
            }
            Err(e) => {
                panic!("Search failed: {:?}", e);
            }
        }
    }
}
