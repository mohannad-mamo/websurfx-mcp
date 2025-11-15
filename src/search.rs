use crate::config::Config;
use crate::errors::{SearxngError, SearxngResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct SearchParams {
    pub q: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
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
    #[allow(dead_code)]
    pub fn new(query: String) -> Self {
        Self {
            q: query,
            format: Some("json".to_string()),
            categories: None,
            engines: None,
            language: None,
            pageno: None,
            time_range: None,
            safesearch: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_categories(mut self, categories: String) -> Self {
        self.categories = Some(categories);
        self
    }

    #[allow(dead_code)]
    pub fn with_engines(mut self, engines: String) -> Self {
        self.engines = Some(engines);
        self
    }

    #[allow(dead_code)]
    pub fn with_language(mut self, language: String) -> Self {
        self.language = Some(language);
        self
    }

    #[allow(dead_code)]
    pub fn with_page(mut self, page: u32) -> Self {
        self.pageno = Some(page);
        self
    }

    #[allow(dead_code)]
    pub fn with_time_range(mut self, time_range: String) -> Self {
        self.time_range = Some(time_range);
        self
    }

    #[allow(dead_code)]
    pub fn with_safe_search(mut self, level: u8) -> Self {
        if level <= 2 {
            self.safesearch = Some(level);
        }
        self
    }

    pub fn validate(&self) -> SearxngResult<()> {
        if self.q.trim().is_empty() {
            return Err(SearxngError::InvalidParameters(
                "Query cannot be empty".to_string(),
            ));
        }

        if let Some(ref time_range) = self.time_range {
            if !["day", "month", "year"].contains(&time_range.as_str()) {
                return Err(SearxngError::InvalidParameters(
                    "Time range must be 'day', 'month', or 'year'".to_string(),
                ));
            }
        }

        if let Some(safesearch) = self.safesearch {
            if safesearch > 2 {
                return Err(SearxngError::InvalidParameters(
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
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub engine: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub score: f64,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct SearchResponse {
    pub query: String,
    pub number_of_results: u64,
    pub results: Vec<SearchResult>,
    #[serde(default)]
    pub engines: Vec<EngineInfo>,
    #[serde(default)]
    pub suggestions: Vec<String>,
    #[serde(default)]
    pub search: SearchInfo,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct EngineInfo {
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
pub struct SearxngClient {
    client: Client,
    config: Config,
}

impl SearxngClient {
    #[allow(dead_code)]
    pub fn new(config: Config) -> SearxngResult<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .user_agent("searxng-mcp/0.1.0")
            .build()
            .map_err(SearxngError::HttpError)?;

        Ok(Self { client, config })
    }

    #[allow(dead_code)]
    pub async fn search(&self, params: SearchParams) -> SearxngResult<SearchResponse> {
        params.validate()?;

        let mut query_params = HashMap::new();
        query_params.insert("q", params.q.clone());
        query_params.insert("format", "json".to_string());

        if let Some(categories) = &params.categories {
            query_params.insert("categories", categories.clone());
        }
        if let Some(engines) = &params.engines {
            query_params.insert("engines", engines.clone());
        }
        if let Some(language) = &params.language {
            query_params.insert("language", language.clone());
        }
        if let Some(pageno) = params.pageno {
            query_params.insert("pageno", pageno.to_string());
        }
        if let Some(time_range) = &params.time_range {
            query_params.insert("time_range", time_range.clone());
        }
        if let Some(safesearch) = params.safesearch {
            query_params.insert("safesearch", safesearch.to_string());
        }

        let response = self
            .client
            .get(self.config.search_url())
            .query(&query_params)
            .send()
            .await
            .map_err(SearxngError::HttpError)?;

        if !response.status().is_success() {
            return Err(SearxngError::ApiError(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        let search_response: SearchResponse =
            response.json().await.map_err(SearxngError::HttpError)?;

        Ok(search_response)
    }

    #[allow(dead_code)]
    pub fn format_results(&self, response: &SearchResponse) -> String {
        if response.results.is_empty() {
            return "No results found for your search query.".to_string();
        }

        let mut formatted = String::new();

        formatted.push_str(&format!("Search Results for: \"{}\"\n", response.query));
        formatted.push_str(&format!("Found {} results\n\n", response.number_of_results));

        let results_to_show = response.results.iter().take(self.config.max_results);

        for (index, result) in results_to_show.enumerate() {
            formatted.push_str(&format!("{}. {}\n", index + 1, result.title));
            formatted.push_str(&format!("   URL: {}\n", result.url));

            if !result.content.is_empty() {
                let content = if result.content.len() > 200 {
                    format!("{}...", &result.content[..200])
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
    pub async fn test_connection(&self) -> SearxngResult<()> {
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

    #[tokio::test]
    async fn test_client_creation() {
        let config = Config::default();
        let client = SearxngClient::new(config);
        assert!(client.is_ok());
    }

    #[tokio::test]
    #[ignore = "requires running SearXNG instance at localhost:8888"]
    async fn test_search_integration() {
        use std::env;

        env::set_var("SEARXNG_BASE_URL", "http://localhost:8888");
        let config = Config::from_env().expect("Failed to load config");
        let client = SearxngClient::new(config).expect("Failed to create client");

        let params = SearchParams::new("rust programming language".to_string())
            .with_language("en".to_string());

        let result = client.search(params).await;

        match result {
            Ok(response) => {
                assert!(!response.query.is_empty());
                println!("Search successful: {} results", response.number_of_results);
            }
            Err(e) => {
                println!("Search failed: {:?}", e);
            }
        }
    }
}
