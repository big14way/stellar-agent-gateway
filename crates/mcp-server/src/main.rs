use anyhow::Result;
use rmcp::{
    handler::server::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    model::{
        CallToolResult, Content, ErrorData as McpError,
        Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    schemars,
    tool, tool_handler, tool_router,
    ServerHandler, ServiceExt,
    transport::stdio,
};
use serde::Deserialize;
use shared::{
    sign_x402_payment,
    x402::{decode_header, encode_header, PaymentRequired},
};

// ── Service registry ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct Service {
    pub name: String,
    pub url: String,
    pub description: String,
    pub price_display: String,
    pub price_amount: String,
}

fn default_services() -> Vec<Service> {
    vec![
        Service {
            name: "get_weather".into(),
            url: "http://localhost:3001/weather".into(),
            description: "Get current weather data for any city".into(),
            price_display: "$0.001 USDC".into(),
            price_amount: "10000".into(),
        },
        Service {
            name: "get_xlm_price".into(),
            url: "http://localhost:3001/price".into(),
            description: "Get current XLM/USDC price and 24h market data".into(),
            price_display: "$0.001 USDC".into(),
            price_amount: "10000".into(),
        },
        Service {
            name: "analyze_account".into(),
            url: "http://localhost:3001/account".into(),
            description: "Analyze a Stellar account's balances and activity".into(),
            price_display: "$0.002 USDC".into(),
            price_amount: "20000".into(),
        },
    ]
}

// ── x402 HTTP client ──────────────────────────────────────────────────────

async fn call_x402_api(
    client: &reqwest::Client,
    url: &str,
    agent_secret: &str,
    rpc_url: &str,
) -> anyhow::Result<serde_json::Value> {
    let resp = client.get(url).send().await
        .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?;

    match resp.status().as_u16() {
        200 => {
            return resp.json::<serde_json::Value>().await
                .map_err(|e| anyhow::anyhow!("Failed to parse response: {}", e));
        }
        402 => {
            let headers = resp.headers().clone();
            let pr_header = headers
                .get("payment-required")
                .or_else(|| headers.get("PAYMENT-REQUIRED"))
                .ok_or_else(|| anyhow::anyhow!("402 missing PAYMENT-REQUIRED header"))?
                .to_str()
                .map_err(|e| anyhow::anyhow!("Invalid header: {}", e))?
                .to_string();

            let payment_required: PaymentRequired = decode_header(&pr_header)
                .map_err(|e| anyhow::anyhow!("Failed to decode payment requirements: {}", e))?;

            let requirements = payment_required
                .accepts
                .into_iter()
                .find(|a| a.network.starts_with("stellar:"))
                .ok_or_else(|| anyhow::anyhow!("No Stellar payment option in 402"))?;

            tracing::info!(
                "Paying {} base units ({}) for {}",
                requirements.amount,
                requirements.network,
                url
            );

            let tx_xdr = sign_x402_payment(agent_secret, &requirements, rpc_url)
                .await
                .map_err(|e| anyhow::anyhow!("Payment signing failed: {}", e))?;

            let payment_payload = serde_json::json!({
                "x402Version": 2,
                "scheme": "exact",
                "network": requirements.network,
                "payload": { "transaction": tx_xdr }
            });

            let encoded_payload = encode_header(&payment_payload);

            let retry = client
                .get(url)
                .header("PAYMENT-SIGNATURE", &encoded_payload)
                .send()
                .await
                .map_err(|e| anyhow::anyhow!("Payment retry failed: {}", e))?;

            let status = retry.status();
            if status.is_success() {
                tracing::info!("x402 payment accepted for {}", url);
                retry.json::<serde_json::Value>().await
                    .map_err(|e| anyhow::anyhow!("Failed to parse paid response: {}", e))
            } else {
                let body = retry.text().await.unwrap_or_default();
                Err(anyhow::anyhow!("Payment rejected (HTTP {}): {}", status, body))
            }
        }
        other => {
            let body = resp.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("Unexpected HTTP {}: {}", other, body))
        }
    }
}

// ── Tool input types ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct CityInput {
    #[schemars(description = "City name to get weather for, e.g. 'Lagos', 'London', 'Tokyo'")]
    city: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct AccountInput {
    #[schemars(description = "Stellar G-address to analyze, e.g. 'GABC...XYZ'")]
    address: String,
}

// ── MCP server struct ─────────────────────────────────────────────────────

#[derive(Clone)]
struct GatewayServer {
    services: Vec<Service>,
    http_client: reqwest::Client,
    agent_secret: String,
    rpc_url: String,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl GatewayServer {
    fn new(agent_secret: String, rpc_url: String) -> Self {
        Self {
            services: default_services(),
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to build HTTP client"),
            agent_secret,
            rpc_url,
            tool_router: Self::tool_router(),
        }
    }

    fn service_url(&self, name: &str) -> String {
        self.services
            .iter()
            .find(|s| s.name == name)
            .map(|s| s.url.clone())
            .unwrap_or_else(|| format!("http://localhost:3001/{}", name.replace('_', "-")))
    }

    #[tool(description = "Get current weather data for any city. \
        COST: $0.001 USDC paid automatically via x402 on Stellar testnet.")]
    async fn get_weather(
        &self,
        Parameters(input): Parameters<CityInput>,
    ) -> Result<CallToolResult, McpError> {
        let url = format!("{}?city={}", self.service_url("get_weather"), input.city);
        match call_x402_api(&self.http_client, &url, &self.agent_secret, &self.rpc_url).await {
            Ok(data) => Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&data).unwrap_or_default(),
            )])),
            Err(e) => Err(McpError::internal_error(e.to_string(), None)),
        }
    }

    #[tool(description = "Get current XLM/USDC price and 24h market data. \
        COST: $0.001 USDC paid automatically via x402 on Stellar testnet.")]
    async fn get_xlm_price(&self) -> Result<CallToolResult, McpError> {
        let url = self.service_url("get_xlm_price");
        match call_x402_api(&self.http_client, &url, &self.agent_secret, &self.rpc_url).await {
            Ok(data) => Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&data).unwrap_or_default(),
            )])),
            Err(e) => Err(McpError::internal_error(e.to_string(), None)),
        }
    }

    #[tool(description = "Analyze a Stellar account: XLM and USDC balances, \
        trustlines, transaction count, and account age. \
        COST: $0.002 USDC paid automatically via x402 on Stellar testnet.")]
    async fn analyze_account(
        &self,
        Parameters(input): Parameters<AccountInput>,
    ) -> Result<CallToolResult, McpError> {
        let url = format!("{}?address={}", self.service_url("analyze_account"), input.address);
        match call_x402_api(&self.http_client, &url, &self.agent_secret, &self.rpc_url).await {
            Ok(data) => Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&data).unwrap_or_default(),
            )])),
            Err(e) => Err(McpError::internal_error(e.to_string(), None)),
        }
    }

    #[tool(description = "List all x402-gated services available in the \
        Stellar Agent Gateway, including names, descriptions, and prices. \
        This tool is FREE — no payment required.")]
    async fn list_services(&self) -> Result<CallToolResult, McpError> {
        let list: Vec<serde_json::Value> = self.services.iter().map(|s| {
            serde_json::json!({
                "name": s.name,
                "url": s.url,
                "description": s.description,
                "price": s.price_display,
                "payment_protocol": "x402 V2 on Stellar testnet",
            })
        }).collect();

        let output = serde_json::json!({
            "services": list,
            "total": list.len(),
            "protocol": "x402 V2",
            "network": "stellar:testnet",
            "token": "USDC"
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&output).unwrap_or_default(),
        )]))
    }
}

#[tool_handler]
impl ServerHandler for GatewayServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "stellar-agent-gateway".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                title: None,
                description: None,
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "Stellar Agent Gateway — 4 tools backed by real x402 micropayments on Stellar.\n\
                 Tools:\n\
                 • get_weather(city) — $0.001 USDC\n\
                 • get_xlm_price() — $0.001 USDC\n\
                 • analyze_account(address) — $0.002 USDC\n\
                 • list_services() — FREE\n\
                 Payments are processed automatically using Soroban auth entry signing."
                    .into(),
            ),
        }
    }
}

// ── main ──────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    // ALL logging to stderr — stdout is for MCP JSON-RPC ONLY
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("stellar_mcp_server=info".parse()?)
                .add_directive("rmcp=warn".parse()?),
        )
        .init();

    let agent_secret = std::env::var("AGENT_SECRET_KEY")
        .map_err(|_| anyhow::anyhow!("AGENT_SECRET_KEY environment variable not set"))?;
    let rpc_url = std::env::var("STELLAR_RPC_URL")
        .unwrap_or_else(|_| "https://soroban-testnet.stellar.org".to_string());

    let server = GatewayServer::new(agent_secret, rpc_url);

    for svc in &server.services {
        tracing::info!("Tool ready: {} ({}) → {}", svc.name, svc.price_display, svc.url);
    }

    tracing::info!("Stellar Agent Gateway MCP server starting on stdio...");

    let service = server
        .serve(stdio())
        .await
        .map_err(|e| anyhow::anyhow!("MCP serve error: {}", e))?;

    tracing::info!("MCP server running — ready for connections");
    service
        .waiting()
        .await
        .map_err(|e| anyhow::anyhow!("Service error: {}", e))?;

    Ok(())
}
