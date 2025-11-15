use crate::errors::{SearxngError, SearxngResult};
use std::env;
use url::Url;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Config {
    pub base_url: Url,
    pub timeout_seconds: u64,
    #[allow(dead_code)]
    pub max_results: usize,
}

impl Config {
    pub fn from_env() -> SearxngResult<Self> {
        let base_url_str =
            env::var("SEARXNG_BASE_URL").map_err(|_| SearxngError::MissingBaseUrl)?;

        let base_url = Url::parse(&base_url_str)
            .map_err(|_| SearxngError::InvalidBaseUrl(base_url_str.clone()))?;

        if base_url.scheme().is_empty() || base_url.host().is_none() {
            return Err(SearxngError::InvalidBaseUrl(base_url_str));
        }

        let timeout_seconds = env::var("SEARXNG_TIMEOUT_SECONDS")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap_or(30);

        let max_results = env::var("SEARXNG_MAX_RESULTS")
            .unwrap_or_else(|_| "20".to_string())
            .parse()
            .unwrap_or(20);

        Ok(Config {
            base_url,
            timeout_seconds,
            max_results,
        })
    }

    #[allow(dead_code)]
    pub fn search_url(&self) -> String {
        let mut url = self.base_url.clone();
        url.set_path("/search");
        url.to_string()
    }

    pub async fn validate(&self) -> SearxngResult<()> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(SearxngError::HttpError)?;

        let response = client
            .get(self.base_url.as_str())
            .send()
            .await
            .map_err(SearxngError::HttpError)?;

        if !response.status().is_success() {
            return Err(SearxngError::InvalidBaseUrl(format!(
                "SearXNG instance at {} returned status: {}",
                self.base_url,
                response.status()
            )));
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            base_url: Url::parse("http://localhost:8888").unwrap(),
            timeout_seconds: 120,
            max_results: 20,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_from_env_missing_url() {
        env::remove_var("SEARXNG_BASE_URL");
        let result = Config::from_env();
        assert!(matches!(result, Err(SearxngError::MissingBaseUrl)));
    }

    #[test]
    fn test_config_from_env_invalid_url() {
        env::set_var("SEARXNG_BASE_URL", "invalid-url");
        let result = Config::from_env();
        assert!(result.is_err());
    }

    #[test]
    fn test_config_from_env_valid() {
        env::set_var("SEARXNG_BASE_URL", "http://localhost:8888");
        let result = Config::from_env();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.base_url.as_str(), "http://localhost:8888/");
    }

    #[test]
    fn test_search_url() {
        let config = Config::default();
        assert_eq!(config.search_url(), "http://localhost:8888/search");
    }
}
