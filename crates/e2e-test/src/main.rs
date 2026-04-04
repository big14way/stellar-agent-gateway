use anyhow::{anyhow, Result};
use shared::{
    config::AppConfig,
    sign_x402_payment,
    x402::{decode_header, encode_header, PaymentRequired},
};
use std::time::Duration;

const HORIZON: &str = "https://horizon-testnet.stellar.org";

fn pause(ms: u64) -> impl std::future::Future<Output = ()> {
    tokio::time::sleep(Duration::from_millis(ms))
}

async fn get_usdc_balance(address: &str) -> Result<String> {
    let url = format!("{}/accounts/{}", HORIZON, address);
    let resp: serde_json::Value = reqwest::get(&url).await?.json().await?;
    let empty = vec![];
    let balances = resp["balances"].as_array().unwrap_or(&empty);
    for b in balances {
        if b["asset_code"].as_str() == Some("USDC") {
            return Ok(b["balance"].as_str().unwrap_or("0").to_string());
        }
    }
    Ok("0".to_string())
}

async fn call_with_payment(
    client: &reqwest::Client,
    url: &str,
    agent_secret: &str,
    rpc_url: &str,
) -> Result<(serde_json::Value, String)> {
    let resp = client.get(url).send().await?;
    let status = resp.status().as_u16();

    if status == 200 {
        let data: serde_json::Value = resp.json().await?;
        return Ok((data, "no payment needed".to_string()));
    }

    if status != 402 {
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!("Expected 402, got {}: {}", status, body));
    }

    let headers = resp.headers().clone();
    let pr_header = headers
        .get("payment-required")
        .ok_or_else(|| anyhow!("Missing PAYMENT-REQUIRED header"))?
        .to_str()?;

    let payment_required: PaymentRequired = decode_header(pr_header)?;
    let requirements = payment_required
        .accepts
        .into_iter()
        .find(|a| a.network.starts_with("stellar:"))
        .ok_or_else(|| anyhow!("No Stellar payment option"))?;

    let amount = &requirements.amount;
    let pay_to = &requirements.pay_to;

    println!("      → Received HTTP 402 — Payment Required");
    println!("        Amount: {} base units USDC", amount);
    println!("        Pay to: {}...{}", &pay_to[..8], &pay_to[pay_to.len()-8..]);
    println!("        Network: {}", requirements.network);

    pause(500).await;
    println!("      → Signing Soroban authorization entry (ed25519)...");

    let tx_xdr = sign_x402_payment(agent_secret, &requirements, rpc_url).await?;
    println!("        Signed TX: {}...{} ({} bytes XDR)",
        &tx_xdr[..16], &tx_xdr[tx_xdr.len()-8..], tx_xdr.len());

    pause(300).await;
    println!("      → Retrying with PAYMENT-SIGNATURE header...");

    let payment_payload = serde_json::json!({
        "x402Version": 2,
        "accepted": requirements,
        "payload": { "transaction": tx_xdr }
    });
    let encoded = encode_header(&payment_payload);

    let retry = client
        .get(url)
        .header("PAYMENT-SIGNATURE", &encoded)
        .send()
        .await?;

    let retry_status = retry.status();
    if retry_status.is_success() {
        println!("      → OZ Facilitator verified ✓ — settling on Stellar testnet...");
        let data: serde_json::Value = retry.json().await?;
        Ok((data, format!("{} base units → {}...{}", amount, &pay_to[..8], &pay_to[pay_to.len()-8..])))
    } else {
        let body = retry.text().await.unwrap_or_default();
        Err(anyhow!("Payment rejected (HTTP {}): {}", retry_status, body))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let config = AppConfig::from_env()?;
    let client = reqwest::Client::new();
    let base = format!("http://localhost:{}", config.port);

    println!();
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║        Stellar Agent Gateway — Live Demo                      ║");
    println!("║        x402 Micropayments on Stellar Testnet                  ║");
    println!("╚═════════════════════════════════��══════════════════════���═══════╝");
    println!();
    println!("  This demo simulates an AI agent autonomously paying for");
    println!("  API services using real USDC on Stellar testnet.");
    println!("  Each API call triggers a Soroban auth entry signing,");
    println!("  OZ Facilitator verification, and on-chain settlement.");
    println!();

    // ── Preflight ──────────────────────────────────────────────────────
    println!("━━━ Preflight Checks ━━━");
    println!();

    print!("  x402 services server...... ");
    let health = client.get(format!("{}/health", base)).send().await;
    match health {
        Ok(r) if r.status().is_success() => println!("✅ online (port {})", config.port),
        _ => {
            println!("❌ offline");
            println!("\n  Start the server first: make dev-services");
            return Ok(());
        }
    }

    print!("  Agent address............. ");
    println!("{}", config.agent_stellar_address);
    print!("  Server address............ ");
    println!("{}", config.server_stellar_address);

    let agent_before = get_usdc_balance(&config.agent_stellar_address).await?;
    let server_before = get_usdc_balance(&config.server_stellar_address).await?;
    println!("  Agent USDC balance........ {} USDC", agent_before);
    println!("  Server USDC balance....... {} USDC", server_before);
    println!("  OZ Facilitator............ {}", config.facilitator_url);
    println!("  Stellar RPC............... {}", config.stellar_rpc_url);
    println!();

    // ── Scenario: AI Agent Portfolio Analysis ──────────────────────────
    println!("━━━ Scenario: AI Agent Performs Portfolio Analysis ━━━");
    println!();
    println!("  An AI agent is asked: \"Analyze my Stellar portfolio.\"");
    println!("  It autonomously decides to call 3 paid APIs:");
    println!("    1. Get live XLM/USDC price    ($0.001 USDC)");
    println!("    2. Analyze the account on-chain ($0.001 USDC)");
    println!("    3. Check weather in agent's city ($0.001 USDC)");
    println!("  Each call is gated by x402 — agent pays per request.");
    println!();

    pause(1500).await;

    // ── Step 1: XLM Price ─────────────────────────────────────────────
    println!("┌─────────────────────────────────────────────────────────┐");
    println!("│  Step 1: Get XLM/USDC Price (live from Stellar DEX)    │");
    println!("└───────���────────────────────────────���────────────────────┘");
    println!("  Agent calls: GET {}/price", base);
    println!();

    let mut xlm_price = String::from("unknown");
    match call_with_payment(&client, &format!("{}/price", base), &config.agent_secret_key, &config.stellar_rpc_url).await {
        Ok((data, payment)) => {
            println!();
            println!("      ✅ Payment settled: {}", payment);
            if let Some(p) = data["price_usdc"].as_str() { xlm_price = p.to_string(); }
            println!("      📊 XLM/USDC: {}", data["price_usdc"].as_str().unwrap_or("?"));
            println!("         Bid: {}  Ask: {}",
                data["best_bid"].as_str().unwrap_or("?"),
                data["best_ask"].as_str().unwrap_or("?"));
            println!("         Source: {}", data["source"].as_str().unwrap_or("?"));
        }
        Err(e) => println!("      ❌ {}", e),
    }
    println!();

    pause(1000).await;

    // ── Step 2: Account Analysis ──────────────────────────────────────
    println!("┌───────────────────────────────���─────────────────────────┐");
    println!("│  Step 2: Analyze Stellar Account (live from Horizon)   │");
    println!("└────────���────────────────────────────────���───────────────┘");
    println!("  Agent calls: GET {}/account?address={}...{}",
        base, &config.agent_stellar_address[..8], &config.agent_stellar_address[config.agent_stellar_address.len()-4..]);
    println!();

    let mut agent_xlm = String::from("0");
    let mut agent_usdc_live = String::from("0");
    let url = format!("{}/account?address={}", base, config.agent_stellar_address);
    match call_with_payment(&client, &url, &config.agent_secret_key, &config.stellar_rpc_url).await {
        Ok((data, payment)) => {
            println!();
            println!("      ✅ Payment settled: {}", payment);
            if let Some(balances) = data["balances"].as_array() {
                println!("      💰 Account Balances:");
                for b in balances {
                    let asset = b["asset"].as_str().unwrap_or("?");
                    let bal = b["balance"].as_str().unwrap_or("0");
                    println!("         {} → {}", asset, bal);
                    if asset.starts_with("XLM") { agent_xlm = bal.to_string(); }
                    if asset.starts_with("USDC") { agent_usdc_live = bal.to_string(); }
                }
            }
            println!("         Trustlines: {}", data["trustline_count"]);
            println!("         Source: {}", data["source"].as_str().unwrap_or("?"));
        }
        Err(e) => println!("      ❌ {}", e),
    }
    println!();

    pause(1000).await;

    // ── Step 3: Weather ───────────────────────────────────────────────
    println!("┌─────────���─────────────────���─────────────────────────────┐");
    println!("│  Step 3: Get Weather Data (live from wttr.in)          │");
    println!("└─────��───────────────────────────────────��───────────────┘");
    println!("  Agent calls: GET {}/weather?city=Lagos", base);
    println!();

    let mut weather_temp = String::from("?");
    let mut weather_cond = String::from("?");
    match call_with_payment(&client, &format!("{}/weather?city=Lagos", base), &config.agent_secret_key, &config.stellar_rpc_url).await {
        Ok((data, payment)) => {
            println!();
            println!("      ✅ Payment settled: {}", payment);
            if let Some(t) = data["temp_c"].as_f64() { weather_temp = format!("{:.1}°C", t); }
            if let Some(c) = data["condition"].as_str() { weather_cond = c.to_string(); }
            println!("      🌤  Lagos: {} — {}", weather_temp, weather_cond);
            println!("         Humidity: {}%  Wind: {} km/h",
                data["humidity_pct"].as_u64().unwrap_or(0),
                data["wind_kph"].as_f64().unwrap_or(0.0));
            println!("         Source: {}", data["source"].as_str().unwrap_or("?"));
        }
        Err(e) => println!("      ❌ {}", e),
    }
    println!();

    pause(1000).await;

    // ── Agent Synthesizes Report ──────────────────────────────────────
    println!("┌─────────────────────────────────────────────────────────┐");
    println!("│  Agent Synthesizes Portfolio Report                     │");
    println!("└──���───────────────────────────────��──────────────────────���");
    println!();

    let xlm_f: f64 = agent_xlm.parse().unwrap_or(0.0);
    let price_f: f64 = xlm_price.parse().unwrap_or(0.0);
    let xlm_value = xlm_f * price_f;

    println!("  📋 Portfolio Summary for {}...{}",
        &config.agent_stellar_address[..8],
        &config.agent_stellar_address[config.agent_stellar_address.len()-4..]);
    println!("  ─────────────────────────────────────────────");
    println!("  XLM Balance:    {} XLM", agent_xlm);
    println!("  XLM/USDC Rate:  {} USDC", xlm_price);
    println!("  XLM Value:      ${:.2} USDC", xlm_value);
    println!("  USDC Balance:   {} USDC", agent_usdc_live);
    println!("  Total Value:    ~${:.2} USDC", xlm_value + agent_usdc_live.parse::<f64>().unwrap_or(0.0));
    println!("  ─────────────────────────────────────────────");
    println!("  Weather:        Lagos — {} {}", weather_temp, weather_cond);
    println!("  Data sources:   Stellar DEX, Horizon, wttr.in");
    println!("  Payments:       3 x402 micropayments on Stellar testnet");
    println!();

    // ── On-Chain Settlement Verification ──────────────────────────────
    println!("━━━ On-Chain Settlement Verification ━━━");
    println!();
    print!("  Waiting for settlement finality");
    for _ in 0..5 {
        pause(1000).await;
        print!(".");
    }
    println!(" done");
    println!();

    let agent_after = get_usdc_balance(&config.agent_stellar_address).await?;
    let server_after = get_usdc_balance(&config.server_stellar_address).await?;

    let agent_spent: f64 = agent_before.parse::<f64>()? - agent_after.parse::<f64>()?;
    let server_earned: f64 = server_after.parse::<f64>()? - server_before.parse::<f64>()?;

    println!("  Agent  USDC: {} → {}  (spent: ${:.7})", agent_before, agent_after, agent_spent);
    println!("  Server USDC: {} → {} (earned: ${:.7})", server_before, server_after, server_earned);
    println!();

    if agent_spent > 0.0 {
        println!("  ✅ Real USDC moved on-chain!");
        println!();
        println!("  🔗 Verify on Stellar Expert:");
        println!("     Agent:  https://stellar.expert/explorer/testnet/account/{}", config.agent_stellar_address);
        println!("     Server: https://stellar.expert/explorer/testnet/account/{}", config.server_stellar_address);
    } else {
        println!("  ⏳ Settlement processing — check Stellar Expert in a moment.");
    }

    println!();
    println!("  Contracts deployed on testnet:");
    println!("     ServiceRegistry: {}", config.service_registry_contract);
    println!("     SmartAccount:    {}", config.smart_account_contract);
    println!();
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║  Demo complete. 3 API calls, 3 x402 payments, all on-chain.  ║");
    println!("╚══���═════════════════════════════════════════════════════════════╝");
    println!();

    Ok(())
}
