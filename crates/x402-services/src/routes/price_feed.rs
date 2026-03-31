use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct PriceResponse {
    pub pair: String,
    pub price_usdc: String,
    pub change_24h_pct: String,
    pub volume_24h_usdc: String,
    pub high_24h: String,
    pub low_24h: String,
    pub timestamp: String,
}

pub async fn get_xlm_price() -> Json<PriceResponse> {
    Json(PriceResponse {
        pair: "XLM/USDC".to_string(),
        price_usdc: "0.1247".to_string(),
        change_24h_pct: "+3.2".to_string(),
        volume_24h_usdc: "48234901".to_string(),
        high_24h: "0.1290".to_string(),
        low_24h: "0.1198".to_string(),
        timestamp: "2026-03-30T12:00:00Z".to_string(),
    })
}
