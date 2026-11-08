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

mod config;
mod errors;
mod search;

use config::Config;
use errors::WebsurfxResult;
use search::{SearchParams, WebsurfxClient};

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SearchRequest {
    /// The search query (max 512 characters).
    pub query: String,
    /// Page number, 1-based. Websurfx supports pages 1-50.
    #[schemars(range(min = 1, max = 50))]
    pub page: Option<u32>,
    /// Safe search level: 0 (off) to 4 (strictest).
    #[schemars(range(min = 0, max = 4))]
    pub safe_search: Option<u8>,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct WebsurfxServer {
    #[allow(dead_code)]
    client: WebsurfxClient,
    tool_router: ToolRouter<WebsurfxServer>,
}

#[tool_router]
impl WebsurfxServer {
    pub async fn new() -> WebsurfxResult<Self> {
        let config = Config::from_env()?;

        config.validate().await?;

        let client = WebsurfxClient::new(config)?;

        Ok(Self {
            client,
            tool_router: Self::tool_router(),
        })
    }

    #[tool(
        description = "Search the web using Websurfx. Returns results from the configured Websurfx instance."
    )]
    async fn websurfx_search(
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

        if let Some(page) = request.page {
            search_params = search_params.with_page(page);
        }

        if let Some(safe_search) = request.safe_search {
            search_params = search_params.with_safe_search(safe_search);
        }

        match self.client.search(search_params).await {
            Ok(response) => {
                let formatted_results = self.client.format_results(&response);
                Ok(CallToolResult::success(vec![Content::text(
                    formatted_results,
                )]))
            }
            Err(e) => {
                eprintln!("Search failed: {:?}", e);
                Err(ErrorData::internal_error(
                    format!("Search failed: {}", e),
                    None,
                ))
            }
        }
    }
}

#[tool_handler]
impl ServerHandler for WebsurfxServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("Use the websurfx_search tool to search the web. Provide a query and optionally a page number (1-50) and safe search level (0-4).".into()),
        }
    }
}

impl Default for WebsurfxServer {
    fn default() -> Self {
        let config = Config::default();
        let client = WebsurfxClient::new(config).expect("Failed to create client");
        Self {
            client,
            tool_router: Self::tool_router(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let websurfx_server = match WebsurfxServer::new().await {
        Ok(server) => server,
        Err(e) => {
            eprintln!("Failed to initialize Websurfx server: {}", e.user_message());
            return Err(e.into());
        }
    };

    let transport = stdio();

    let service = websurfx_server.serve(transport).await?;
    service.waiting().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_request_schema_only_advertises_supported_parameters() {
        let schema = schemars::schema_for!(SearchRequest);
        let value = serde_json::to_value(&schema).unwrap();
        let properties = value
            .get("properties")
            .and_then(|p| p.as_object())
            .expect("schema should have properties");

        let mut keys: Vec<&str> = properties.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(keys, ["page", "query", "safe_search"]);

        let page = &properties["page"];
        assert_eq!(page.get("minimum").and_then(|v| v.as_u64()), Some(1));
        assert_eq!(page.get("maximum").and_then(|v| v.as_u64()), Some(50));

        let safe_search = &properties["safe_search"];
        assert_eq!(safe_search.get("minimum").and_then(|v| v.as_u64()), Some(0));
        let safe_search_max = safe_search.get("maximum").and_then(|v| v.as_u64());
        assert_eq!(safe_search_max, Some(4));
    }
}
