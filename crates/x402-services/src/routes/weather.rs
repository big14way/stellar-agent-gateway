use axum::{extract::Query, Json};
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Deserialize)]
pub struct WeatherQuery {
    pub city: String,
}

#[derive(Serialize)]
pub struct WeatherResponse {
    pub city: String,
    pub temp_c: f64,
    pub condition: String,
    pub humidity_pct: u32,
    pub wind_kph: f64,
    pub source: String,
    pub timestamp: String,
}

async fn fetch_live_weather(city: &str) -> Option<WeatherResponse> {
    // wttr.in is a free, no-auth weather API
    let url = format!("https://wttr.in/{}?format=j1", city);
    let resp = reqwest::Client::new()
        .get(&url)
        .header("User-Agent", "stellar-agent-gateway/0.1")
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let data: serde_json::Value = resp.json().await.ok()?;

    let current = data["current_condition"].as_array()?.first()?;
    let temp_c: f64 = current["temp_C"].as_str()?.parse().ok()?;
    let humidity: u32 = current["humidity"].as_str()?.parse().ok()?;
    let wind_kph: f64 = current["windspeedKmph"].as_str()?.parse().ok()?;
    let condition = current["weatherDesc"]
        .as_array()?
        .first()?["value"]
        .as_str()?
        .to_string();

    Some(WeatherResponse {
        city: city.to_string(),
        temp_c,
        condition,
        humidity_pct: humidity,
        wind_kph,
        source: "wttr.in (live)".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

pub async fn get_weather(Query(q): Query<WeatherQuery>) -> Json<WeatherResponse> {
    match fetch_live_weather(&q.city).await {
        Some(weather) => Json(weather),
        None => {
            warn!("Failed to fetch live weather for {}, returning fallback", q.city);
            Json(WeatherResponse {
                city: q.city,
                temp_c: 28.5,
                condition: "Data unavailable".to_string(),
                humidity_pct: 0,
                wind_kph: 0.0,
                source: "fallback (API unavailable)".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            })
        }
    }
}
