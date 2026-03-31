use axum::{extract::Query, Json};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct WeatherQuery { pub city: String }

#[derive(Serialize)]
pub struct WeatherResponse {
    pub city: String,
    pub temp_c: f32,
    pub condition: String,
    pub humidity_pct: u32,
    pub wind_kph: f32,
    pub uv_index: u32,
    pub timestamp: String,
}

pub async fn get_weather(Query(q): Query<WeatherQuery>) -> Json<WeatherResponse> {
    Json(WeatherResponse {
        city: q.city,
        temp_c: 28.5,
        condition: "Partly cloudy".to_string(),
        humidity_pct: 72,
        wind_kph: 14.0,
        uv_index: 6,
        timestamp: chrono_now(),
    })
}

fn chrono_now() -> String {
    // Simple timestamp without chrono dependency
    "2026-03-30T12:00:00Z".to_string()
}
