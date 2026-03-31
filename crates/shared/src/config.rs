#[derive(Debug, Clone)]
pub struct AppConfig {
    pub stellar_rpc_url: String,
    pub horizon_url: String,
    pub network_passphrase: String,
    pub facilitator_url: String,
    pub facilitator_api_key: String,
    pub server_secret_key: String,
    pub server_stellar_address: String,
    pub agent_secret_key: String,
    pub agent_stellar_address: String,
    pub usdc_contract: String,
    pub service_registry_contract: String,
    pub smart_account_contract: String,
    pub port: u16,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        Ok(Self {
            stellar_rpc_url: std::env::var("STELLAR_RPC_URL")
                .unwrap_or_else(|_| "https://soroban-testnet.stellar.org".into()),
            horizon_url: std::env::var("HORIZON_URL")
                .unwrap_or_else(|_| "https://horizon-testnet.stellar.org".into()),
            network_passphrase: std::env::var("STELLAR_NETWORK_PASSPHRASE")
                .unwrap_or_else(|_| "Test SDF Network ; September 2015".into()),
            facilitator_url: std::env::var("FACILITATOR_URL")
                .unwrap_or_else(|_| "https://channels.openzeppelin.com/x402/testnet".into()),
            facilitator_api_key: std::env::var("FACILITATOR_API_KEY").unwrap_or_default(),
            server_secret_key: std::env::var("SERVER_SECRET_KEY").unwrap_or_default(),
            server_stellar_address: std::env::var("SERVER_STELLAR_ADDRESS").unwrap_or_default(),
            agent_secret_key: std::env::var("AGENT_SECRET_KEY").unwrap_or_default(),
            agent_stellar_address: std::env::var("AGENT_STELLAR_ADDRESS").unwrap_or_default(),
            usdc_contract: std::env::var("USDC_CONTRACT")
                .unwrap_or_else(|_| "CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA".into()),
            service_registry_contract: std::env::var("SERVICE_REGISTRY_CONTRACT")
                .unwrap_or_default(),
            smart_account_contract: std::env::var("SMART_ACCOUNT_CONTRACT")
                .unwrap_or_default(),
            port: std::env::var("PORT").ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3001),
        })
    }
}
