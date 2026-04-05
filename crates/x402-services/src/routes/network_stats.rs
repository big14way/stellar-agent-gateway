use axum::Json;
use serde::Serialize;
use tracing::warn;

const HORIZON: &str = "https://horizon-testnet.stellar.org";

#[derive(Serialize)]
pub struct NetworkStatsResponse {
    pub latest_ledger: u32,
    pub base_fee_stroops: String,
    pub protocol_version: u32,
    pub history_latest_ledger: u32,
    pub network: String,
    pub source: String,
    pub timestamp: String,
}

async fn fetch_network_stats() -> Option<NetworkStatsResponse> {
    // Get fee stats
    let fee_url = format!("{}/fee_stats", HORIZON);
    let fee_resp: serde_json::Value = reqwest::get(&fee_url).await.ok()?.json().await.ok()?;

    // Get latest ledger
    let ledger_url = format!("{}/ledgers?limit=1&order=desc", HORIZON);
    let ledger_resp: serde_json::Value = reqwest::get(&ledger_url).await.ok()?.json().await.ok()?;
    let latest = ledger_resp["_embedded"]["records"].as_array()?.first()?;

    Some(NetworkStatsResponse {
        latest_ledger: latest["sequence"].as_u64()? as u32,
        base_fee_stroops: fee_resp["last_ledger_base_fee"].as_str()?.to_string(),
        protocol_version: latest["protocol_version"].as_u64()? as u32,
        history_latest_ledger: latest["sequence"].as_u64()? as u32,
        network: "Stellar Testnet".to_string(),
        source: "Stellar Horizon (live)".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

pub async fn get_network_stats() -> Json<NetworkStatsResponse> {
    match fetch_network_stats().await {
        Some(resp) => Json(resp),
        None => {
            warn!("Failed to fetch network stats");
            Json(NetworkStatsResponse {
                latest_ledger: 0,
                base_fee_stroops: "100".to_string(),
                protocol_version: 0,
                history_latest_ledger: 0,
                network: "Stellar Testnet".to_string(),
                source: "Stellar Horizon (unavailable)".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            })
        }
    }
}
