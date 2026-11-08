use crate::errors::{WebsurfxError, WebsurfxResult};
use std::env;
use std::fmt;
use url::Url;

const WEBSURFX_BASE_URL_KEY: &str = "WEBSURFX_BASE_URL";
const WEBSURFX_TIMEOUT_SECONDS_KEY: &str = "WEBSURFX_TIMEOUT_SECONDS";
const WEBSURFX_MAX_RESULTS_KEY: &str = "WEBSURFX_MAX_RESULTS";
const WEBSURFX_VALIDATION_TIMEOUT_SECONDS_KEY: &str = "WEBSURFX_VALIDATION_TIMEOUT_SECONDS";

const WEBSURFX_DEFAULT_BASE_URL: &str = "http://localhost:8080";
const WEBSURFX_DEFAULT_TIMEOUT_SECONDS: u64 = 30;
const WEBSURFX_DEFAULT_MAX_RESULTS: usize = 20;
const WEBSURFX_DEFAULT_VALIDATION_TIMEOUT_SECONDS: u64 = 10;

fn parse_env_var_or_default<T>(key: &str, default: T) -> Result<T, String>
where
    T: std::str::FromStr,
    T::Err: fmt::Display,
{
    match env::var(key) {
        Ok(value) if !value.is_empty() => value
            .parse()
            .map_err(|e| format!("Invalid value for environment variable '{}': {}", key, e)),
        Ok(value) if value.is_empty() => {
            Err(format!("Empty value for environment variable '{}'", key))
        }
        _ => Ok(default),
    }
}

/// Configuration for the Websurfx MCP server.
///
/// The configuration can be loaded from environment variables using [`Config::from_env()`]
/// or created with default values using [`Config::default()`].
///
/// # Environment Variables
/// - `WEBSURFX_BASE_URL`: Base URL of the Websurfx instance (default: http://localhost:8080)
/// - `WEBSURFX_TIMEOUT_SECONDS`: Request timeout in seconds (default: 30)
/// - `WEBSURFX_MAX_RESULTS`: Maximum number of search results to return (default: 20)
/// - `WEBSURFX_VALIDATION_TIMEOUT_SECONDS`: Validation timeout in seconds (default: 10)
#[derive(Debug, Clone)]
pub struct Config {
    /// Base URL of the Websurfx instance
    pub base_url: Url,
    /// Timeout for HTTP requests in seconds
    pub timeout_seconds: u64,
    /// Maximum number of search results to display
    pub max_results: usize,
    /// Timeout for validation requests in seconds
    pub validation_timeout_seconds: u64,
}

impl Config {
    /// Creates a new Config from environment variables.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The base URL is invalid or uses an unsupported scheme
    /// - Environment variables contain invalid values that cannot be parsed
    pub fn from_env() -> WebsurfxResult<Self> {
        let base_url_str = env::var(WEBSURFX_BASE_URL_KEY)
            .unwrap_or_else(|_| WEBSURFX_DEFAULT_BASE_URL.to_string());

        let base_url = Url::parse(&base_url_str)
            .map_err(|_| WebsurfxError::InvalidBaseUrl(base_url_str.clone()))?;

        if base_url.host().is_none() {
            return Err(WebsurfxError::InvalidBaseUrl(base_url_str));
        }

        match base_url.scheme() {
            "http" | "https" => {}
            _ => {
                return Err(WebsurfxError::InvalidBaseUrl(format!(
                    "URL must use http or https scheme, got: {}",
                    base_url_str
                )))
            }
        }

        let timeout_seconds = parse_env_var_or_default(
            WEBSURFX_TIMEOUT_SECONDS_KEY,
            WEBSURFX_DEFAULT_TIMEOUT_SECONDS,
        )
        .map_err(WebsurfxError::InvalidEnvVar)?;

        let max_results =
            parse_env_var_or_default(WEBSURFX_MAX_RESULTS_KEY, WEBSURFX_DEFAULT_MAX_RESULTS)
                .map_err(WebsurfxError::InvalidEnvVar)?;

        let validation_timeout_seconds = parse_env_var_or_default(
            WEBSURFX_VALIDATION_TIMEOUT_SECONDS_KEY,
            WEBSURFX_DEFAULT_VALIDATION_TIMEOUT_SECONDS,
        )
        .map_err(WebsurfxError::InvalidEnvVar)?;

        Ok(Config {
            base_url,
            timeout_seconds,
            max_results,
            validation_timeout_seconds,
        })
    }

    /// Returns the full search API URL.
    pub fn search_url(&self) -> String {
        let mut url = self.base_url.clone();
        url.set_path("/search");
        url.to_string()
    }

    /// Validates the configuration by attempting to connect to the Websurfx instance.
    ///
    /// # Errors
    /// Returns an error if the Websurfx instance is unreachable or returns a non-success status.
    pub async fn validate(&self) -> WebsurfxResult<()> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(
                self.validation_timeout_seconds,
            ))
            .build()
            .map_err(WebsurfxError::HttpError)?;

        let response = client
            .get(self.base_url.as_str())
            .send()
            .await
            .map_err(WebsurfxError::HttpError)?;

        if !response.status().is_success() {
            return Err(WebsurfxError::InvalidBaseUrl(format!(
                "Websurfx instance at {} returned status: {}",
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
            base_url: Url::parse(WEBSURFX_DEFAULT_BASE_URL).unwrap(),
            timeout_seconds: WEBSURFX_DEFAULT_TIMEOUT_SECONDS,
            max_results: WEBSURFX_DEFAULT_MAX_RESULTS,
            validation_timeout_seconds: WEBSURFX_DEFAULT_VALIDATION_TIMEOUT_SECONDS,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests modify environment variables and should be run sequentially
    // to avoid interference. Run with: cargo test config::tests -- --test-threads=1

    #[test]
    fn test_default_matches_constants() {
        let config = Config::default();
        assert_eq!(config.timeout_seconds, WEBSURFX_DEFAULT_TIMEOUT_SECONDS);
        assert_eq!(config.max_results, WEBSURFX_DEFAULT_MAX_RESULTS);
        assert_eq!(
            config.validation_timeout_seconds,
            WEBSURFX_DEFAULT_VALIDATION_TIMEOUT_SECONDS
        );
        assert_eq!(
            config.base_url.as_str(),
            format!("{}/", WEBSURFX_DEFAULT_BASE_URL)
        );
    }

    #[test]
    fn test_search_url_generation() {
        let test_cases = vec![
            ("http://localhost:8080", "http://localhost:8080/search"),
            ("http://localhost:8080/", "http://localhost:8080/search"), // with trailing slash
            (
                "http://localhost:8080/prefix",
                "http://localhost:8080/search",
            ), // with existing path
        ];

        for (base_url_str, expected_url) in test_cases {
            let config = Config {
                base_url: Url::parse(base_url_str).unwrap(),
                ..Config::default()
            };
            assert_eq!(
                config.search_url(),
                expected_url,
                "Failed for base URL: {}",
                base_url_str
            );
        }
    }

    type ConfigChecker = Box<dyn Fn(&str, WebsurfxResult<Config>)>;

    #[test]
    fn test_from_env_scenarios() {
        struct TestCase<'a> {
            name: &'a str,
            vars_to_set: Vec<(&'a str, &'a str)>,
            checker: ConfigChecker,
        }

        let test_cases = vec![
            TestCase {
                name: "default_config",
                vars_to_set: vec![],
                checker: Box::new(|name, result| {
                    assert!(result.is_ok(), "Test case '{}' failed", name);
                    let config = result.unwrap();
                    assert_eq!(
                        config.timeout_seconds, WEBSURFX_DEFAULT_TIMEOUT_SECONDS,
                        "Test case '{}' failed",
                        name
                    );
                    assert_eq!(
                        config.max_results, WEBSURFX_DEFAULT_MAX_RESULTS,
                        "Test case '{}' failed",
                        name
                    );
                    assert_eq!(
                        config.validation_timeout_seconds,
                        WEBSURFX_DEFAULT_VALIDATION_TIMEOUT_SECONDS,
                        "Test case '{}' failed",
                        name
                    );
                }),
            },
            TestCase {
                name: "invalid_base_url_scheme",
                vars_to_set: vec![(WEBSURFX_BASE_URL_KEY, "ftp://localhost:8080")],
                checker: Box::new(|name, result| {
                    assert!(result.is_err(), "Test case '{}' failed", name);
                    let msg = result.err().unwrap().to_string();
                    assert!(
                        msg.contains("URL must use http or https scheme"),
                        "Test case '{}' failed",
                        name
                    );
                }),
            },
            TestCase {
                name: "valid_http_scheme",
                vars_to_set: vec![(WEBSURFX_BASE_URL_KEY, "http://localhost:8080")],
                checker: Box::new(|name, result| {
                    assert!(result.is_ok(), "Test case '{}' failed", name);
                    assert_eq!(
                        result.unwrap().base_url.as_str(),
                        "http://localhost:8080/",
                        "Test case '{}' failed",
                        name
                    );
                }),
            },
            TestCase {
                name: "valid_https_scheme",
                vars_to_set: vec![(WEBSURFX_BASE_URL_KEY, "https://my.websurfx.instance")],
                checker: Box::new(|name, result| {
                    assert!(result.is_ok(), "Test case '{}' failed", name);
                    assert_eq!(
                        result.unwrap().base_url.as_str(),
                        "https://my.websurfx.instance/",
                        "Test case '{}' failed",
                        name
                    );
                }),
            },
            TestCase {
                name: "url_without_host",
                vars_to_set: vec![(WEBSURFX_BASE_URL_KEY, "http://")],
                checker: Box::new(|name, result| {
                    assert!(result.is_err(), "Test case '{}' failed", name)
                }),
            },
            TestCase {
                name: "invalid_timeout_value",
                vars_to_set: vec![(WEBSURFX_TIMEOUT_SECONDS_KEY, "not-a-number")],
                checker: Box::new(|name, result| {
                    assert!(result.is_err(), "Test case '{}' failed", name)
                }),
            },
            TestCase {
                name: "valid_custom_timeout",
                vars_to_set: vec![(WEBSURFX_TIMEOUT_SECONDS_KEY, "60")],
                checker: Box::new(|name, result| {
                    assert!(result.is_ok(), "Test case '{}' failed", name);
                    assert_eq!(
                        result.unwrap().timeout_seconds,
                        60,
                        "Test case '{}' failed",
                        name
                    );
                }),
            },
            TestCase {
                name: "empty_timeout_returns_error",
                vars_to_set: vec![(WEBSURFX_TIMEOUT_SECONDS_KEY, "")],
                checker: Box::new(|name, result| {
                    assert!(result.is_err(), "Test case '{}' failed", name);
                    assert_eq!(
                        result.unwrap_err().user_message(),
                        "Invalid environment variable: Empty value for environment variable 'WEBSURFX_TIMEOUT_SECONDS'",
                        "Test case '{}' failed",
                        name
                    );
                }),
            },
            TestCase {
                name: "invalid_max_results_value",
                vars_to_set: vec![(WEBSURFX_MAX_RESULTS_KEY, "invalid")],
                checker: Box::new(|name, result| {
                    assert!(result.is_err(), "Test case '{}' failed", name)
                }),
            },
            TestCase {
                name: "valid_custom_max_results",
                vars_to_set: vec![(WEBSURFX_MAX_RESULTS_KEY, "50")],
                checker: Box::new(|name, result| {
                    assert!(result.is_ok(), "Test case '{}' failed", name);
                    assert_eq!(
                        result.unwrap().max_results,
                        50,
                        "Test case '{}' failed",
                        name
                    );
                }),
            },
            TestCase {
                name: "invalid_validation_timeout_value",
                vars_to_set: vec![(WEBSURFX_VALIDATION_TIMEOUT_SECONDS_KEY, "invalid")],
                checker: Box::new(|name, result| {
                    assert!(result.is_err(), "Test case '{}' failed", name)
                }),
            },
            TestCase {
                name: "valid_custom_validation_timeout",
                vars_to_set: vec![(WEBSURFX_VALIDATION_TIMEOUT_SECONDS_KEY, "15")],
                checker: Box::new(|name, result| {
                    assert!(result.is_ok(), "Test case '{}' failed", name);
                    assert_eq!(
                        result.unwrap().validation_timeout_seconds,
                        15,
                        "Test case '{}' failed",
                        name
                    );
                }),
            },
        ];

        for case in test_cases {
            env::remove_var(WEBSURFX_BASE_URL_KEY);
            env::remove_var(WEBSURFX_TIMEOUT_SECONDS_KEY);
            env::remove_var(WEBSURFX_MAX_RESULTS_KEY);
            env::remove_var(WEBSURFX_VALIDATION_TIMEOUT_SECONDS_KEY);

            for (key, value) in &case.vars_to_set {
                env::set_var(key, value);
            }

            let result = Config::from_env();
            (case.checker)(case.name, result);
        }

        env::remove_var(WEBSURFX_BASE_URL_KEY);
        env::remove_var(WEBSURFX_TIMEOUT_SECONDS_KEY);
        env::remove_var(WEBSURFX_MAX_RESULTS_KEY);
        env::remove_var(WEBSURFX_VALIDATION_TIMEOUT_SECONDS_KEY);
    }
}
