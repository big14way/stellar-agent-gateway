use anyhow::{anyhow, Result};
use shared::{
    config::AppConfig,
    sign_x402_payment,
    x402::{decode_header, encode_header, PaymentRequired},
};

const HORIZON: &str = "https://horizon-testnet.stellar.org";

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
    // Step 1: GET without payment
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

    // Step 2: Decode payment requirements from header
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

    // Step 3: Sign Soroban auth entry
    let tx_xdr = sign_x402_payment(agent_secret, &requirements, rpc_url).await?;

    // Step 4: Build payment payload and retry
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
        let data: serde_json::Value = retry.json().await?;
        Ok((data, format!("{} base units to {}", amount, &pay_to[..12])))
    } else {
        let body = retry.text().await.unwrap_or_default();
        Err(anyhow!("Payment rejected (HTTP {}): {}", retry_status, body))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter("e2e_test=info")
        .init();

    let config = AppConfig::from_env()?;
    let client = reqwest::Client::new();
    let base = format!("http://localhost:{}", config.port);

    println!();
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║   Stellar Agent Gateway — End-to-End Payment Test   ║");
    println!("╚══════════════════════════════════════════════════════╝");
    println!();

    // Check server health
    print!("[1/7] Checking x402 server... ");
    let health = client.get(format!("{}/health", base)).send().await;
    match health {
        Ok(r) if r.status().is_success() => println!("✅ online"),
        _ => {
            println!("❌ offline");
            println!("      Run: make dev-services");
            return Ok(());
        }
    }

    // Show starting balances
    print!("[2/7] Agent USDC balance...  ");
    let agent_before = get_usdc_balance(&config.agent_stellar_address).await?;
    println!("{} USDC", agent_before);

    print!("[3/7] Server USDC balance... ");
    let server_before = get_usdc_balance(&config.server_stellar_address).await?;
    println!("{} USDC", server_before);

    // Test 1: Weather API ($0.001 USDC)
    println!();
    println!("─── Test: Weather API ($0.001 USDC) ───");
    print!("[4/7] Calling /weather?city=Lagos with x402 payment... ");
    let url = format!("{}/weather?city=Lagos", base);
    match call_with_payment(&client, &url, &config.agent_secret_key, &config.stellar_rpc_url).await {
        Ok((data, payment_info)) => {
            println!("✅");
            println!("      Payment: {}", payment_info);
            println!("      Response: {}", serde_json::to_string_pretty(&data)?);
        }
        Err(e) => {
            println!("❌ {}", e);
        }
    }

    // Test 2: XLM Price ($0.001 USDC)
    println!();
    println!("─── Test: XLM Price Feed ($0.001 USDC) ───");
    print!("[5/7] Calling /price with x402 payment... ");
    let url = format!("{}/price", base);
    match call_with_payment(&client, &url, &config.agent_secret_key, &config.stellar_rpc_url).await {
        Ok((data, payment_info)) => {
            println!("✅");
            println!("      Payment: {}", payment_info);
            println!("      Response: {}", serde_json::to_string_pretty(&data)?);
        }
        Err(e) => {
            println!("❌ {}", e);
        }
    }

    // Test 3: Account Analyzer ($0.002 USDC)
    println!();
    println!("─── Test: Account Analyzer ($0.002 USDC) ───");
    print!("[6/7] Calling /account with x402 payment... ");
    let url = format!("{}/account?address={}", base, config.server_stellar_address);
    match call_with_payment(&client, &url, &config.agent_secret_key, &config.stellar_rpc_url).await {
        Ok((data, payment_info)) => {
            println!("✅");
            println!("      Payment: {}", payment_info);
            println!("      Response: {}", serde_json::to_string_pretty(&data)?);
        }
        Err(e) => {
            println!("❌ {}", e);
        }
    }

    // Check final balances
    println!();
    println!("─── Final Balances ───");
    print!("[7/7] Checking on-chain balances... ");
    // Wait a moment for settlement
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    let agent_after = get_usdc_balance(&config.agent_stellar_address).await?;
    let server_after = get_usdc_balance(&config.server_stellar_address).await?;
    println!("done");
    println!();
    println!("      Agent  USDC: {} → {}", agent_before, agent_after);
    println!("      Server USDC: {} → {}", server_before, server_after);

    let agent_diff: f64 = agent_before.parse::<f64>()? - agent_after.parse::<f64>()?;
    if agent_diff > 0.0 {
        println!();
        println!("      💰 Total spent: {:.7} USDC on-chain", agent_diff);
        println!();
        println!("      View agent transactions:");
        println!("      https://stellar.expert/explorer/testnet/account/{}", config.agent_stellar_address);
    } else {
        println!();
        println!("      ⏳ Settlement may still be processing — check Stellar Expert in a moment.");
    }

    println!();
    println!("══════════════════════════════════════════════════════");
    println!("  End-to-end test complete.");
    println!("══════════════════════════════════════════════════════");
    println!();

    Ok(())
}
