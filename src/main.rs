use anyhow::Result;
use rmcp::{
    handler::server::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    model::{
        CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    schemars::JsonSchema,
    service::ServiceExt,
    tool, tool_handler, tool_router,
    transport::io::stdio,
    ErrorData, ServerHandler,
};
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{error, info};

mod config;
mod errors;
mod search;

use config::Config;
use errors::SearxngResult;
use search::{SearchParams, SearxngClient};

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SearchRequest {
    pub query: String,
    pub categories: Option<String>,
    pub engines: Option<String>,
    pub language: Option<String>,
    pub page: Option<u32>,
    pub time_range: Option<String>,
    pub safe_search: Option<u8>,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct SearxngServer {
    #[allow(dead_code)]
    client: SearxngClient,
    tool_router: ToolRouter<SearxngServer>,
}

#[tool_router]
impl SearxngServer {
    pub async fn new() -> SearxngResult<Self> {
        let config = Config::from_env()?;

        info!(
            "Testing connection to SearXNG instance at {}",
            config.base_url
        );
        config.validate().await?;
        info!("Successfully connected to SearXNG instance");

        let client = SearxngClient::new(config)?;

        Ok(Self {
            client,
            tool_router: Self::tool_router(),
        })
    }

    #[tool(
        description = "Search the web using SearXNG. Provides comprehensive search results from multiple search engines."
    )]
    async fn searxng_search(
        &self,
        Parameters(request): Parameters<SearchRequest>,
    ) -> std::result::Result<CallToolResult, ErrorData> {
        if request.query.trim().is_empty() {
            return Err(ErrorData::invalid_params(
                "Query cannot be empty".to_string(),
                None,
            ));
        }

        let mut search_params = SearchParams::new(request.query);

        if let Some(categories) = request.categories {
            if !categories.trim().is_empty() {
                search_params = search_params.with_categories(categories);
            }
        }

        if let Some(engines) = request.engines {
            if !engines.trim().is_empty() {
                search_params = search_params.with_engines(engines);
            }
        }

        if let Some(language) = request.language {
            if !language.trim().is_empty() {
                search_params = search_params.with_language(language);
            }
        }

        if let Some(page) = request.page {
            if page > 0 {
                search_params = search_params.with_page(page);
            }
        }

        if let Some(time_range) = request.time_range {
            if !time_range.trim().is_empty() {
                search_params = search_params.with_time_range(time_range);
            }
        }

        if let Some(safe_search) = request.safe_search {
            if safe_search <= 2 {
                search_params = search_params.with_safe_search(safe_search);
            }
        }

        match self.client.search(search_params).await {
            Ok(response) => {
                let formatted_results = self.client.format_results(&response);
                Ok(CallToolResult::success(vec![Content::text(
                    formatted_results,
                )]))
            }
            Err(e) => {
                error!("Search failed: {:?}", e);
                Err(ErrorData::internal_error(
                    format!("Search failed: {}", e),
                    None,
                ))
            }
        }
    }
}

#[tool_handler]
impl ServerHandler for SearxngServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("Use the searxng_search tool to search the web. Provide a query and optionally specify categories, engines, language, page number, time range, and safe search level.".into()),
        }
    }
}

impl Default for SearxngServer {
    fn default() -> Self {
        let config = Config::default();
        let client = SearxngClient::new(config).expect("Failed to create client");
        Self {
            client,
            tool_router: Self::tool_router(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            env::var("RUST_LOG").unwrap_or_else(|_| "searxng_mcp=debug,rmcp=debug".to_string()),
        )
        .init();

    info!("Starting SearXNG MCP Server");

    let searxng_server = match SearxngServer::new().await {
        Ok(server) => server,
        Err(e) => {
            error!("Failed to initialize SearXNG server: {}", e.user_message());
            return Err(e.into());
        }
    };

    info!("SearXNG MCP Server initialized successfully");

    let transport = stdio();

    let service = searxng_server.serve(transport).await?;
    service.waiting().await?;

    info!("SearXNG MCP Server shutting down");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires running SearXNG instance"]
    async fn test_server_initialization() {
        std::env::set_var("SEARXNG_BASE_URL", "http://localhost:8888");
        let result = SearxngServer::new().await;

        match result {
            Ok(_) => println!("Server initialized successfully"),
            Err(e) => println!("Server initialization failed: {:?}", e),
        }
    }

    #[test]
    fn test_search_request_validation() {
        let request = SearchRequest {
            query: "".to_string(),
            categories: None,
            engines: None,
            language: None,
            page: None,
            time_range: None,
            safe_search: None,
        };

        assert!(request.query.trim().is_empty());

        let request = SearchRequest {
            query: "rust programming".to_string(),
            categories: Some("general,news".to_string()),
            engines: None,
            language: Some("en".to_string()),
            page: Some(1),
            time_range: Some("month".to_string()),
            safe_search: Some(1),
        };

        assert!(!request.query.trim().is_empty());
        assert_eq!(request.categories, Some("general,news".to_string()));
        assert_eq!(request.language, Some("en".to_string()));
        assert_eq!(request.page, Some(1));
        assert_eq!(request.time_range, Some("month".to_string()));
        assert_eq!(request.safe_search, Some(1));
    }
}
