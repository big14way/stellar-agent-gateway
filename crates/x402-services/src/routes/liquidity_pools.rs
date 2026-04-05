use axum::Json;
use serde::Serialize;
use tracing::warn;

const HORIZON: &str = "https://horizon-testnet.stellar.org";

#[derive(Serialize)]
pub struct PoolAsset {
    pub asset: String,
    pub amount: String,
}

#[derive(Serialize)]
pub struct LiquidityPool {
    pub id: String,
    pub fee_bp: u32,
    pub total_shares: String,
    pub total_trustlines: String,
    pub reserves: Vec<PoolAsset>,
}

#[derive(Serialize)]
pub struct LiquidityPoolsResponse {
    pub pools: Vec<LiquidityPool>,
    pub count: usize,
    pub source: String,
    pub timestamp: String,
}

async fn fetch_pools() -> Option<LiquidityPoolsResponse> {
    let url = format!("{}/liquidity_pools?limit=10&order=desc", HORIZON);
    let resp = reqwest::get(&url).await.ok()?;
    if !resp.status().is_success() { return None; }
    let data: serde_json::Value = resp.json().await.ok()?;

    let records = data["_embedded"]["records"].as_array()?;
    let pools: Vec<LiquidityPool> = records.iter().filter_map(|r| {
        let reserves = r["reserves"].as_array()?.iter().filter_map(|res| {
            let asset_str = res["asset"].as_str()?;
            let asset = if asset_str == "native" {
                "XLM".to_string()
            } else {
                let parts: Vec<&str> = asset_str.split(':').collect();
                if parts.len() >= 1 { parts[0].to_string() } else { asset_str.to_string() }
            };
            Some(PoolAsset {
                asset,
                amount: res["amount"].as_str()?.to_string(),
            })
        }).collect();

        Some(LiquidityPool {
            id: r["id"].as_str().map(|s| format!("{}...{}", &s[..8], &s[s.len()-8..])).unwrap_or_default(),
            fee_bp: r["fee_bp"].as_u64().unwrap_or(30) as u32,
            total_shares: r["total_shares"].as_str()?.to_string(),
            total_trustlines: r["total_trustlines"].as_u64().unwrap_or(0).to_string(),
            reserves,
        })
    }).collect();

    let count = pools.len();
    Some(LiquidityPoolsResponse {
        pools,
        count,
        source: "Stellar AMM (Horizon /liquidity_pools)".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

pub async fn get_liquidity_pools() -> Json<LiquidityPoolsResponse> {
    match fetch_pools().await {
        Some(resp) => Json(resp),
        None => {
            warn!("Failed to fetch liquidity pools");
            Json(LiquidityPoolsResponse {
                pools: vec![],
                count: 0,
                source: "Stellar AMM (unavailable)".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            })
        }
    }
}
