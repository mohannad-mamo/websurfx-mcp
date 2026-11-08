use anyhow::{bail, Context, Result};
use axum::{routing::get, Router};
use rmcp::{
    handler::server::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    model::{
        CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    schemars::JsonSchema,
    service::ServiceExt,
    tool, tool_handler, tool_router,
    transport::{
        io::stdio, streamable_http_server::session::local::LocalSessionManager,
        StreamableHttpServerConfig, StreamableHttpService,
    },
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
    fn from_config(config: Config) -> WebsurfxResult<Self> {
        let client = WebsurfxClient::new(config)?;

        Ok(Self {
            client,
            tool_router: Self::tool_router(),
        })
    }

    pub async fn new() -> WebsurfxResult<Self> {
        let config = Config::from_env()?;

        config.validate().await?;

        Self::from_config(config)
    }

    #[tool(
        description = "Search the web using Websurfx. Returns results from the configured Websurfx instance.",
        annotations(
            title = "Websurfx Web Search",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
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

        match self.client.search_with_retry(search_params).await {
            Ok(response) => {
                let formatted_results = self.client.format_results(&response);
                Ok(CallToolResult::success(vec![Content::text(
                    formatted_results,
                )]))
            }
            Err(e) => {
                eprintln!("Search failed: {:?}", e);
                Err(e.into())
            }
        }
    }
}

#[tool_handler]
impl ServerHandler for WebsurfxServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: env!("CARGO_PKG_NAME").to_string(),
                title: Some("Websurfx MCP Server".to_string()),
                version: env!("CARGO_PKG_VERSION").to_string(),
                icons: None,
                website_url: Some(
                    "https://github.com/mohannad-mamo/websurfx-mcp".to_string(),
                ),
            },
            instructions: Some("Use the websurfx_search tool to search the web. Provide a query and optionally a page number (1-50) and safe search level (0-4).".into()),
        }
    }
}

impl Default for WebsurfxServer {
    fn default() -> Self {
        Self::from_config(Config::default()).expect("Failed to create client")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransportMode {
    Stdio,
    Http,
}

fn parse_transport(value: &str) -> Result<TransportMode> {
    match value.to_ascii_lowercase().as_str() {
        "stdio" => Ok(TransportMode::Stdio),
        "http" => Ok(TransportMode::Http),
        other => bail!(
            "Unsupported MCP_TRANSPORT '{}': expected 'stdio' or 'http'",
            other
        ),
    }
}

fn transport_from_env() -> Result<TransportMode> {
    let value = std::env::var("MCP_TRANSPORT").unwrap_or_else(|_| "stdio".to_string());
    parse_transport(&value)
}

async fn serve_stdio() -> Result<()> {
    let server = WebsurfxServer::new().await?;
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

async fn serve_http() -> Result<()> {
    let config = Config::from_env()?;

    // Do not refuse to start while Websurfx is still booting. The health
    // endpoint should come up even if the search backend is not ready yet.
    if let Err(e) = config.validate().await {
        eprintln!(
            "Warning: Websurfx is not reachable at startup: {}",
            e.user_message()
        );
    }

    let app = http_app(WebsurfxServer::from_config(config)?);

    let bind_addr = std::env::var("MCP_HTTP_BIND").unwrap_or_else(|_| "0.0.0.0:8001".to_string());
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .with_context(|| format!("failed to bind MCP HTTP server to {}", bind_addr))?;

    eprintln!(
        "Websurfx MCP server listening on http://{}/mcp (stateless)",
        bind_addr
    );

    axum::serve(listener, app).await?;
    Ok(())
}

fn http_app(server: WebsurfxServer) -> Router {
    let mcp_service: StreamableHttpService<WebsurfxServer, LocalSessionManager> =
        StreamableHttpService::new(
            move || Ok(server.clone()),
            Default::default(),
            StreamableHttpServerConfig {
                stateful_mode: false,
                sse_keep_alive: None,
            },
        );

    Router::new()
        .route("/healthz", get(healthz))
        .nest_service("/mcp", mcp_service)
}

async fn healthz() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() -> Result<()> {
    match transport_from_env()? {
        TransportMode::Stdio => serve_stdio().await,
        TransportMode::Http => serve_http().await,
    }
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

    #[test]
    fn transport_mode_parsing() {
        assert_eq!(parse_transport("stdio").unwrap(), TransportMode::Stdio);
        assert_eq!(parse_transport("HTTP").unwrap(), TransportMode::Http);
        assert!(parse_transport("sse").is_err());
    }

    #[test]
    fn tool_is_annotated_read_only_and_open_world() {
        let router = WebsurfxServer::tool_router();
        let tool = &router
            .map
            .get("websurfx_search")
            .expect("websurfx_search tool is registered")
            .attr;
        let annotations = tool.annotations.as_ref().expect("tool annotations");

        assert_eq!(annotations.title.as_deref(), Some("Websurfx Web Search"));
        assert_eq!(annotations.read_only_hint, Some(true));
        assert_eq!(annotations.destructive_hint, Some(false));
        assert_eq!(annotations.idempotent_hint, Some(false));
        assert_eq!(annotations.open_world_hint, Some(true));
    }

    #[test]
    fn server_info_identifies_websurfx_mcp() {
        let info = WebsurfxServer::default().get_info();

        assert_eq!(info.protocol_version, ProtocolVersion::LATEST);
        assert_eq!(info.server_info.name, "websurfx-mcp");
        assert_eq!(info.server_info.version, env!("CARGO_PKG_VERSION"));
    }

    #[tokio::test]
    async fn http_transport_serves_health_and_initialize() {
        let server = WebsurfxServer::from_config(Config::default()).unwrap();
        let app = http_app(server);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let client = reqwest::Client::new();

        let health = client
            .get(format!("http://{addr}/healthz"))
            .send()
            .await
            .unwrap();
        assert_eq!(health.status(), reqwest::StatusCode::OK);
        assert_eq!(health.text().await.unwrap(), "ok");

        let response = client
            .post(format!("http://{addr}/mcp"))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .body(
                r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"0"}}}"#,
            )
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), reqwest::StatusCode::OK);
        let body = response.text().await.unwrap();
        let payload = body
            .lines()
            .find_map(|line| line.strip_prefix("data: "))
            .unwrap_or(&body);
        let json: serde_json::Value = serde_json::from_str(payload).unwrap();

        assert_eq!(json["result"]["serverInfo"]["name"], "websurfx-mcp");
        assert_eq!(json["result"]["protocolVersion"], "2025-03-26");

        handle.abort();
    }
}
