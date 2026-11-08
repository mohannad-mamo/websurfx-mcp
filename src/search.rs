use crate::config::Config;
use crate::errors::{WebsurfxError, WebsurfxResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const USER_AGENT: &str = "websurfx-mcp/0.2";
const DEFAULT_FORMAT: &str = "true";
const MIN_PAGE: u32 = 1;
const MAX_PAGE: u32 = 50;
const MAX_SAFE_SEARCH_LEVEL: u8 = 4;
const MAX_QUERY_LENGTH: usize = 512;
const MAX_CONTENT_LENGTH: usize = 200;
const MAX_ERROR_BODY_LENGTH: usize = 500;
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

/// Ensures a section is separated by exactly one blank line.
fn begin_notice(formatted: &mut String) {
    if !formatted.ends_with("\n\n") {
        formatted.push('\n');
    }
}

/// Truncates on a UTF-8 character boundary. Slicing by bytes panics for
/// multi-byte scripts like Arabic, Hebrew, CJK, etc.
fn truncate_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() > max_chars {
        let mut truncated: String = value.chars().take(max_chars).collect();
        truncated.push_str("...");
        truncated
    } else {
        value.to_string()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchParams {
    pub q: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safesearch: Option<u8>,
}

impl SearchParams {
    pub fn new(query: String) -> Self {
        Self {
            q: query,
            json: Some(DEFAULT_FORMAT.to_string()),
            page: None,
            safesearch: None,
        }
    }

    pub fn with_page(mut self, page: u32) -> Self {
        self.page = Some(page);
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

        if self.q.chars().count() > MAX_QUERY_LENGTH {
            return Err(WebsurfxError::InvalidParameters(format!(
                "Query must be at most {} characters",
                MAX_QUERY_LENGTH
            )));
        }

        if let Some(page) = self.page {
            if !(MIN_PAGE..=MAX_PAGE).contains(&page) {
                return Err(WebsurfxError::InvalidParameters(format!(
                    "Page must be between {} and {}",
                    MIN_PAGE, MAX_PAGE
                )));
            }
        }

        if let Some(safesearch) = self.safesearch {
            if safesearch > MAX_SAFE_SEARCH_LEVEL {
                return Err(WebsurfxError::InvalidParameters(format!(
                    "Safe search level must be between 0 and {}",
                    MAX_SAFE_SEARCH_LEVEL
                )));
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
    #[serde(default, alias = "relevanceScore")]
    pub score: f64,
}

/// Error information reported by Websurfx for an upstream engine.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct EngineErrorInfo {
    #[serde(default)]
    pub error: String,
    #[serde(default)]
    pub engine: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct SearchResponse {
    #[serde(default)]
    pub query: String,
    #[serde(default)]
    pub results: Vec<SearchResult>,
    // Websurfx may spell these camelCase (v1.31.x) or snake_case depending on
    // the version/config, and engine_errors_info is sometimes null.
    #[serde(default, alias = "engineErrorsInfo")]
    pub engine_errors_info: Option<Vec<EngineErrorInfo>>,
    #[serde(default)]
    pub disallowed: bool,
    #[serde(default)]
    pub filtered: bool,
    #[serde(default, alias = "safeSearchLevel")]
    pub safe_search_level: u8,
    #[serde(default, alias = "noEnginesSelected")]
    pub no_engines_selected: bool,
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

        // Websurfx only returns JSON when json=true is set. Using
        // format=json returns HTML instead.
        params.json = Some(DEFAULT_FORMAT.to_string());

        let response = self
            .client
            .get(self.config.search_url())
            .query(&params)
            .send()
            .await
            .map_err(WebsurfxError::HttpError)?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = truncate_chars(
                &response.text().await.unwrap_or_default(),
                MAX_ERROR_BODY_LENGTH,
            );
            return Err(WebsurfxError::HttpStatusError { status, body });
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
            WebsurfxError::HttpStatusError { status, .. } => {
                // Retry on server errors and rate limiting, but not on other
                // client errors such as a malformed query.
                (500..=599).contains(status) || *status == 429
            }
            WebsurfxError::ApiError(_) => false,
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
        let engine_errors = response.engine_errors_info.as_deref().unwrap_or_default();
        let mut formatted = String::new();

        if response.results.is_empty() {
            formatted.push_str(&format!("No results found for: \"{}\"\n", response.query));
        } else {
            let total = response.results.len();
            let noun = if total == 1 { "result" } else { "results" };
            formatted.push_str(&format!("Search Results for: \"{}\"\n", response.query));
            if total > self.config.max_results {
                formatted.push_str(&format!(
                    "Found {} {} (showing the first {})\n\n",
                    total, noun, self.config.max_results
                ));
            } else {
                formatted.push_str(&format!("Found {} {}\n\n", total, noun));
            }

            let results_to_show = response.results.iter().take(self.config.max_results);

            for (index, result) in results_to_show.enumerate() {
                formatted.push_str(&format!("{}. {}\n", index + 1, result.title));
                formatted.push_str(&format!("   URL: {}\n", result.url));

                if !result.content.is_empty() {
                    let content = truncate_chars(&result.content, MAX_CONTENT_LENGTH);
                    formatted.push_str(&format!("   {}\n", content));
                }

                if !result.engine.is_empty() {
                    formatted.push_str(&format!("   Source: {}\n", result.engine));
                }

                formatted.push('\n');
            }
        }

        self.append_response_notices(&mut formatted, response, engine_errors);

        formatted
    }

    fn append_response_notices(
        &self,
        formatted: &mut String,
        response: &SearchResponse,
        engine_errors: &[EngineErrorInfo],
    ) {
        if response.disallowed {
            begin_notice(formatted);
            formatted.push_str(&format!(
                "Websurfx disallowed this query at safe search level {} (server-side blocklist).\n",
                response.safe_search_level
            ));
        }

        if response.filtered {
            begin_notice(formatted);
            formatted
                .push_str("Websurfx reported the results as filtered (server-side filtering).\n");
        }

        if response.no_engines_selected {
            begin_notice(formatted);
            formatted.push_str("Websurfx has no search engines selected/enabled.\n");
        }

        if !engine_errors.is_empty() {
            begin_notice(formatted);
            formatted.push_str("Engine warnings:\n");
            for info in engine_errors {
                let engine = if info.engine.is_empty() {
                    "unknown engine"
                } else {
                    &info.engine
                };
                let error = if info.error.is_empty() {
                    "unknown error"
                } else {
                    &info.error
                };
                formatted.push_str(&format!("- {}: {}\n", engine, error));
            }
        }
    }

    #[allow(dead_code)]
    pub async fn test_connection(&self) -> WebsurfxResult<()> {
        self.config.validate().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EMPTY_ENGINE_ERROR_JSON: &str =
        include_str!("../tests/fixtures/search/empty_engine_error.json");
    const EMPTY_ENGINE_ERROR_EXPECTED: &str =
        include_str!("../tests/fixtures/search/empty_engine_error.expected.txt");
    const CAMEL_CASE_NULL_ERRORS_JSON: &str =
        include_str!("../tests/fixtures/search/camel_case_null_errors.json");
    const RESULT_ALIASES_JSON: &str = include_str!("../tests/fixtures/search/result_aliases.json");
    const DISALLOWED_JSON: &str = include_str!("../tests/fixtures/search/disallowed.json");
    const DISALLOWED_EXPECTED: &str =
        include_str!("../tests/fixtures/search/disallowed.expected.txt");
    const FILTERED_NO_ENGINES_JSON: &str =
        include_str!("../tests/fixtures/search/filtered_no_engines.json");
    const FILTERED_NO_ENGINES_EXPECTED: &str =
        include_str!("../tests/fixtures/search/filtered_no_engines.expected.txt");
    const RESULTS_WITH_ENGINE_WARNING_JSON: &str =
        include_str!("../tests/fixtures/search/results_with_engine_warning.json");
    const RESULTS_WITH_ENGINE_WARNING_EXPECTED: &str =
        include_str!("../tests/fixtures/search/results_with_engine_warning.expected.txt");
    const TRUNCATED_RESULTS_JSON: &str =
        include_str!("../tests/fixtures/search/truncated_results.json");
    const TRUNCATED_RESULTS_EXPECTED: &str =
        include_str!("../tests/fixtures/search/truncated_results.expected.txt");

    fn parse_fixture(json: &str, query: &str) -> SearchResponse {
        let mut response: SearchResponse = serde_json::from_str(json).unwrap();
        // Websurfx does not echo the query, so the client fills it in.
        response.query = query.to_string();
        response
    }

    fn format_fixture(json: &str, expected: &str, query: &str, config: Config) {
        let client = WebsurfxClient::new(config).unwrap();
        let response = parse_fixture(json, query);
        assert_eq!(client.format_results(&response), expected);
    }

    #[test]
    fn test_search_params_builder() {
        let params = SearchParams::new("test query".to_string())
            .with_page(2)
            .with_safe_search(1);

        assert_eq!(params.q, "test query");
        assert_eq!(params.page, Some(2));
        assert_eq!(params.safesearch, Some(1));
    }

    #[test]
    fn test_search_params_validation() {
        let valid_params = SearchParams::new("test query".to_string())
            .with_page(50)
            .with_safe_search(4);
        assert!(valid_params.validate().is_ok());

        let invalid_safe_search = SearchParams::new("test query".to_string()).with_safe_search(5);
        assert!(invalid_safe_search.validate().is_err());

        let page_too_low = SearchParams::new("test query".to_string()).with_page(0);
        assert!(page_too_low.validate().is_err());

        let page_too_high = SearchParams::new("test query".to_string()).with_page(51);
        assert!(page_too_high.validate().is_err());

        let empty_query = SearchParams::new("   ".to_string());
        assert!(empty_query.validate().is_err());

        let max_length_query = SearchParams::new("x".repeat(MAX_QUERY_LENGTH));
        assert!(max_length_query.validate().is_ok());

        let too_long_query = SearchParams::new("x".repeat(MAX_QUERY_LENGTH + 1));
        assert!(too_long_query.validate().is_err());
    }

    #[test]
    fn test_search_params_serialization() {
        let params = SearchParams::new("test query".to_string())
            .with_page(2)
            .with_safe_search(1);

        let serialized = serde_json::to_string(&params).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();

        assert_eq!(parsed.get("q").unwrap(), "test query");
        assert_eq!(parsed.get("json").unwrap(), "true");
        assert_eq!(parsed.get("page").unwrap(), 2);
        assert_eq!(parsed.get("safesearch").unwrap(), 1);

        assert!(parsed.get("pageno").is_none());
        assert!(parsed.get("categories").is_none());
        assert!(parsed.get("engines").is_none());
        assert!(parsed.get("language").is_none());
        assert!(parsed.get("time_range").is_none());

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
        let server_error = WebsurfxError::HttpStatusError {
            status: 503,
            body: "unavailable".to_string(),
        };
        assert!(WebsurfxClient::is_retryable_error(&server_error));

        let rate_limited = WebsurfxError::HttpStatusError {
            status: 429,
            body: "slow down".to_string(),
        };
        assert!(WebsurfxClient::is_retryable_error(&rate_limited));

        let bad_request = WebsurfxError::HttpStatusError {
            status: 400,
            body: "bad query".to_string(),
        };
        assert!(!WebsurfxClient::is_retryable_error(&bad_request));

        let api_error = WebsurfxError::ApiError("server busy".to_string());
        assert!(!WebsurfxClient::is_retryable_error(&api_error));

        let param_error = WebsurfxError::InvalidParameters("invalid query".to_string());
        assert!(!WebsurfxClient::is_retryable_error(&param_error));

        let no_results_error = WebsurfxError::NoResults;
        assert!(!WebsurfxClient::is_retryable_error(&no_results_error));
    }

    #[test]
    fn test_parse_websurfx_payload_with_snake_case_fields() {
        let response: SearchResponse = serde_json::from_str(EMPTY_ENGINE_ERROR_JSON).unwrap();

        assert_eq!(response.safe_search_level, 0);
        let errors = response.engine_errors_info.as_deref().unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].engine, "duckduckgo");
        assert_eq!(errors[0].error, "EmptyResultSet");
    }

    #[test]
    fn test_parse_websurfx_payload_with_camel_case_fields_and_null_errors() {
        let response: SearchResponse = serde_json::from_str(CAMEL_CASE_NULL_ERRORS_JSON).unwrap();

        assert!(response.engine_errors_info.is_none());
        assert!(response.filtered);
        assert!(response.no_engines_selected);
        assert_eq!(response.safe_search_level, 3);
    }

    #[test]
    fn test_parse_search_result_aliases() {
        let response: SearchResponse = serde_json::from_str(RESULT_ALIASES_JSON).unwrap();
        let result = &response.results[0];

        assert_eq!(result.content, "A language empowering everyone.");
        assert_eq!(result.engine, "wikipedia-en, duckduckgo");
        assert_eq!(result.score, 0.75);
    }

    #[test]
    fn test_format_surfaces_engine_errors_instead_of_silent_empty() {
        format_fixture(
            EMPTY_ENGINE_ERROR_JSON,
            EMPTY_ENGINE_ERROR_EXPECTED,
            "rust",
            Config::default(),
        );
    }

    #[test]
    fn test_format_surfaces_status_flags() {
        format_fixture(
            DISALLOWED_JSON,
            DISALLOWED_EXPECTED,
            "blocked query",
            Config::default(),
        );
        format_fixture(
            FILTERED_NO_ENGINES_JSON,
            FILTERED_NO_ENGINES_EXPECTED,
            "filtered",
            Config::default(),
        );
    }

    #[test]
    fn test_format_keeps_results_and_adds_engine_warnings() {
        format_fixture(
            RESULTS_WITH_ENGINE_WARNING_JSON,
            RESULTS_WITH_ENGINE_WARNING_EXPECTED,
            "rust",
            Config::default(),
        );
    }

    #[test]
    fn test_format_reports_when_results_are_truncated() {
        format_fixture(
            TRUNCATED_RESULTS_JSON,
            TRUNCATED_RESULTS_EXPECTED,
            "rust",
            Config {
                max_results: 1,
                ..Config::default()
            },
        );
    }

    #[test]
    fn test_truncate_chars_respects_character_boundaries() {
        assert_eq!(truncate_chars("short", 10), "short");
        assert_eq!(truncate_chars("مرحبا بالعالم", 5), "مرحبا...");
        assert_eq!(truncate_chars("αβγδεζ", 3), "αβγ...");
    }

    #[tokio::test]
    async fn test_search_integration() {
        use std::env;
        use url::Url;

        if env::var("WEBSURFX_LIVE").unwrap_or_default() != "1" {
            return;
        }

        let base_url =
            env::var("WEBSURFX_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
        let config = Config {
            base_url: Url::parse(&base_url).expect("invalid WEBSURFX_BASE_URL"),
            ..Config::default()
        };
        let client = WebsurfxClient::new(config).expect("Failed to create client");

        let params = SearchParams::new("rust programming language".to_string()).with_page(1);
        let response = client.search(params).await.expect("Search failed");

        assert!(!response.query.is_empty());
        println!("Search successful: {} results", response.results.len());
    }
}
