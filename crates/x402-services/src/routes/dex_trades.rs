use axum::{extract::Query, Json};
use serde::{Deserialize, Serialize};
use tracing::warn;

const HORIZON: &str = "https://horizon-testnet.stellar.org";

#[derive(Deserialize)]
pub struct DexTradesQuery {
    /// Base asset (e.g. "native" for XLM, or "USDC:GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5")
    pub base: Option<String>,
    /// Counter asset
    pub counter: Option<String>,
    /// Number of trades to return (default 10)
    pub limit: Option<u32>,
}

#[derive(Serialize)]
pub struct Trade {
    pub id: String,
    pub base_amount: String,
    pub counter_amount: String,
    pub price: String,
    pub base_asset: String,
    pub counter_asset: String,
    pub ledger_close_time: String,
}

#[derive(Serialize)]
pub struct DexTradesResponse {
    pub trades: Vec<Trade>,
    pub count: usize,
    pub source: String,
    pub timestamp: String,
}

fn parse_asset_params(asset: &str) -> (String, String) {
    if asset == "native" || asset == "XLM" {
        return ("native".into(), String::new());
    }
    let parts: Vec<&str> = asset.split(':').collect();
    if parts.len() == 2 {
        (format!("credit_alphanum4&base_asset_code={}&base_asset_issuer={}", parts[0], parts[1]),
         String::new())
    } else {
        ("native".into(), String::new())
    }
}

async fn fetch_trades(limit: u32) -> Option<DexTradesResponse> {
    // Fetch recent trades for XLM/USDC on the Stellar DEX
    let url = format!(
        "{}/trades?base_asset_type=native&counter_asset_type=credit_alphanum4&counter_asset_code=USDC&counter_asset_issuer=GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5&order=desc&limit={}",
        HORIZON, limit
    );
    let resp = reqwest::get(&url).await.ok()?;
    if !resp.status().is_success() { return None; }
    let data: serde_json::Value = resp.json().await.ok()?;

    let records = data["_embedded"]["records"].as_array()?;
    let trades: Vec<Trade> = records.iter().filter_map(|r| {
        Some(Trade {
            id: r["id"].as_str()?.to_string(),
            base_amount: r["base_amount"].as_str()?.to_string(),
            counter_amount: r["counter_amount"].as_str()?.to_string(),
            price: format!("{}", r["price"]["n"].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0)
                / r["price"]["d"].as_str().unwrap_or("1").parse::<f64>().unwrap_or(1.0)),
            base_asset: "XLM".to_string(),
            counter_asset: "USDC".to_string(),
            ledger_close_time: r["ledger_close_time"].as_str()?.to_string(),
        })
    }).collect();

    let count = trades.len();
    Some(DexTradesResponse {
        trades,
        count,
        source: "Stellar DEX (Horizon /trades)".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

pub async fn get_dex_trades(Query(q): Query<DexTradesQuery>) -> Json<DexTradesResponse> {
    let limit = q.limit.unwrap_or(10).min(25);
    match fetch_trades(limit).await {
        Some(resp) => Json(resp),
        None => {
            warn!("Failed to fetch DEX trades");
            Json(DexTradesResponse {
                trades: vec![],
                count: 0,
                source: "Stellar DEX (unavailable)".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            })
        }
    }
}
