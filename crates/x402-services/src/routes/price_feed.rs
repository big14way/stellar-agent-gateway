use axum::Json;
use serde::Serialize;
use tracing::warn;

const HORIZON: &str = "https://horizon-testnet.stellar.org";
// Stellar DEX orderbook: XLM sold for USDC
const USDC_ASSET: &str = "USDC";
const USDC_ISSUER: &str = "GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5";

#[derive(Serialize)]
pub struct PriceResponse {
    pub pair: String,
    pub price_usdc: String,
    pub source: String,
    pub best_bid: String,
    pub best_ask: String,
    pub spread_pct: String,
    pub timestamp: String,
}

async fn fetch_live_price() -> Option<PriceResponse> {
    let url = format!(
        "{}/order_book?selling_asset_type=native&buying_asset_type=credit_alphanum4&buying_asset_code={}&buying_asset_issuer={}&limit=1",
        HORIZON, USDC_ASSET, USDC_ISSUER
    );
    let resp = reqwest::get(&url).await.ok()?;
    let data: serde_json::Value = resp.json().await.ok()?;

    let best_bid = data["bids"]
        .as_array()?
        .first()?["price"]
        .as_str()?
        .to_string();
    let best_ask = data["asks"]
        .as_array()?
        .first()?["price"]
        .as_str()?
        .to_string();

    let bid_f: f64 = best_bid.parse().ok()?;
    let ask_f: f64 = best_ask.parse().ok()?;
    let mid = (bid_f + ask_f) / 2.0;
    let spread = ((ask_f - bid_f) / mid * 100.0).abs();

    Some(PriceResponse {
        pair: "XLM/USDC".to_string(),
        price_usdc: format!("{:.7}", mid),
        source: "Stellar DEX (Horizon orderbook)".to_string(),
        best_bid,
        best_ask,
        spread_pct: format!("{:.4}", spread),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

pub async fn get_xlm_price() -> Json<PriceResponse> {
    match fetch_live_price().await {
        Some(price) => Json(price),
        None => {
            warn!("Failed to fetch live XLM price, returning fallback");
            Json(PriceResponse {
                pair: "XLM/USDC".to_string(),
                price_usdc: "0.1247".to_string(),
                source: "fallback (Horizon unavailable)".to_string(),
                best_bid: "n/a".to_string(),
                best_ask: "n/a".to_string(),
                spread_pct: "n/a".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            })
        }
    }
}
