use axum::{middleware, routing::get, Router};
use tower_http::cors::CorsLayer;
use shared::AppConfig;

mod middleware_x402;
mod routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("x402_services=info".parse()?),
        )
        .init();

    let config = AppConfig::from_env()?;
    let port = config.port;

    tracing::info!("x402 services starting on port {}", port);
    tracing::info!("Server address: {}", config.server_stellar_address);
    tracing::info!("USDC contract:  {}", config.usdc_contract);
    tracing::info!("Facilitator:    {}", config.facilitator_url);

    // Health route — no payment gate
    let public_routes = Router::new()
        .route("/health", get(|| async { "Stellar Agent Gateway x402 Services OK" }));

    // Paid routes — x402 middleware
    let paid_routes = Router::new()
        .route("/weather",         get(routes::weather::get_weather))
        .route("/price",           get(routes::price_feed::get_xlm_price))
        .route("/account",         get(routes::account_analyzer::analyze_account))
        .route("/dex-trades",      get(routes::dex_trades::get_dex_trades))
        .route("/liquidity-pools", get(routes::liquidity_pools::get_liquidity_pools))
        .route("/network-stats",   get(routes::network_stats::get_network_stats))
        .layer(middleware::from_fn_with_state(
            config.clone(),
            middleware_x402::x402_payment_guard,
        ));

    let app = Router::new()
        .merge(public_routes)
        .merge(paid_routes)
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
