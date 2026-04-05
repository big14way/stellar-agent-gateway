use anyhow::{anyhow, Result};
use shared::{
    config::AppConfig,
    sign_x402_payment,
    x402::{decode_header, encode_header, PaymentRequired},
};
use std::time::Duration;

const HORIZON: &str = "https://horizon-testnet.stellar.org";
const COST_PER_CALL: f64 = 0.001; // $0.001 USDC per API call

fn pause(ms: u64) -> impl std::future::Future<Output = ()> {
    tokio::time::sleep(Duration::from_millis(ms))
}

async fn get_usdc_balance(address: &str) -> Result<f64> {
    let url = format!("{}/accounts/{}", HORIZON, address);
    let resp: serde_json::Value = reqwest::get(&url).await?.json().await?;
    let empty = vec![];
    let balances = resp["balances"].as_array().unwrap_or(&empty);
    for b in balances {
        if b["asset_code"].as_str() == Some("USDC") {
            return Ok(b["balance"].as_str().unwrap_or("0").parse().unwrap_or(0.0));
        }
    }
    Ok(0.0)
}

async fn call_with_payment(
    client: &reqwest::Client,
    url: &str,
    agent_secret: &str,
    rpc_url: &str,
) -> Result<serde_json::Value> {
    let resp = client.get(url).send().await?;
    if resp.status().as_u16() == 200 {
        return resp.json().await.map_err(|e| anyhow!("{}", e));
    }
    if resp.status().as_u16() != 402 {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!("Expected 402, got {}: {}", status, body));
    }

    let headers = resp.headers().clone();
    let pr_header = headers.get("payment-required")
        .ok_or_else(|| anyhow!("Missing PAYMENT-REQUIRED header"))?.to_str()?;
    let payment_required: PaymentRequired = decode_header(pr_header)?;
    let requirements = payment_required.accepts.into_iter()
        .find(|a| a.network.starts_with("stellar:"))
        .ok_or_else(|| anyhow!("No Stellar payment option"))?;

    println!("      → 402 received. Signing Soroban auth entry...");
    let tx_xdr = sign_x402_payment(agent_secret, &requirements, rpc_url).await?;
    println!("      → Signed. Retrying with payment...");

    let payment_payload = serde_json::json!({
        "x402Version": 2,
        "accepted": requirements,
        "payload": { "transaction": tx_xdr }
    });
    let encoded = encode_header(&payment_payload);
    let retry = client.get(url).header("PAYMENT-SIGNATURE", &encoded).send().await?;

    let retry_status = retry.status();
    if retry_status.is_success() {
        println!("      → Facilitator verified ✓ — settled on Stellar testnet");
        retry.json().await.map_err(|e| anyhow!("{}", e))
    } else {
        let body = retry.text().await.unwrap_or_default();
        Err(anyhow!("Payment rejected ({}): {}", retry_status, body))
    }
}

struct Budget {
    limit: f64,
    spent: f64,
    calls: u32,
}

impl Budget {
    fn new(limit: f64) -> Self { Self { limit, spent: 0.0, calls: 0 } }

    fn check(&self, cost: f64) -> bool { self.spent + cost <= self.limit }

    fn spend(&mut self, cost: f64) {
        self.spent += cost;
        self.calls += 1;
    }

    fn display(&self) -> String {
        format!("${:.4} / ${:.4} ({} calls)", self.spent, self.limit, self.calls)
    }

    fn remaining(&self) -> f64 { self.limit - self.spent }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let config = AppConfig::from_env()?;
    let client = reqwest::Client::new();
    let base = format!("http://localhost:{}", config.port);

    println!();
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║   Stellar Agent Gateway — Live Demo                          ║");
    println!("║   x402 Micropayments · Soroban Smart Contracts · Stellar     ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();
    println!("  An AI agent autonomously pays for Stellar data services");
    println!("  using real USDC on testnet. Every payment settles on-chain.");
    println!();

    // ── Preflight ─────────────────────────────────────────────────────
    println!("━━━ Preflight ━━━");
    println!();

    print!("  x402 services............ ");
    let health = client.get(format!("{}/health", base)).send().await;
    match health {
        Ok(r) if r.status().is_success() => println!("✅ online (port {})", config.port),
        _ => { println!("❌ offline — run: make dev-services"); return Ok(()); }
    }

    let agent_before = get_usdc_balance(&config.agent_stellar_address).await?;
    let server_before = get_usdc_balance(&config.server_stellar_address).await?;
    println!("  Agent.................... {} (${:.7} USDC)", &config.agent_stellar_address[..12], agent_before);
    println!("  Server................... {} (${:.7} USDC)", &config.server_stellar_address[..12], server_before);

    // ── Session Budget ────────────────────────────────────────────────
    let session_budget = 0.01; // $0.01 USDC max for this session
    let mut budget = Budget::new(session_budget);

    println!();
    println!("  ┌─────────────────────────────────────────────┐");
    println!("  │  Session Budget: ${:.4} USDC               │", session_budget);
    println!("  │  Cost per call:  ${:.4} USDC               │", COST_PER_CALL);
    println!("  │  Max calls:      {}                          │", (session_budget / COST_PER_CALL) as u32);
    println!("  └─────────────────────────────────────────────┘");
    println!();

    pause(1000).await;

    // ── Scenario ──────────────────────────────────────────────────────
    println!("━━━ Scenario: Comprehensive Stellar Market Analysis ━━━");
    println!();
    println!("  Agent is asked: \"Give me a full Stellar market report.\"");
    println!("  It plans 5 autonomous paid API calls:");
    println!("    1. XLM/USDC price (Stellar DEX)");
    println!("    2. Recent DEX trades");
    println!("    3. Liquidity pool reserves");
    println!("    4. Network stats (ledger, fees)");
    println!("    5. Agent account analysis (Horizon)");
    println!();

    pause(1500).await;

    // ── Tool 1: XLM Price ─────────────────────────────────────────────
    println!("┌─────────────────────────────────────────────────────────┐");
    println!("│  1/5  XLM/USDC Price (Stellar DEX Orderbook)           │");
    println!("└─────────────────────────────────────────────────────────┘");
    if !budget.check(COST_PER_CALL) {
        println!("  ⛔ Budget exceeded! Remaining: ${:.4}", budget.remaining());
    } else {
        println!("  Budget: {} → spending ${:.4}...", budget.display(), COST_PER_CALL);
        match call_with_payment(&client, &format!("{}/price", base), &config.agent_secret_key, &config.stellar_rpc_url).await {
            Ok(data) => {
                budget.spend(COST_PER_CALL);
                println!("      ✅ XLM/USDC: {}  (source: {})",
                    data["price_usdc"].as_str().unwrap_or("?"),
                    data["source"].as_str().unwrap_or("?"));
            }
            Err(e) => println!("      ❌ {}", e),
        }
    }
    println!();
    pause(800).await;

    // ── Tool 2: DEX Trades ────────────────────────────────────────────
    println!("┌─────────────────────────────────────────────────────────┐");
    println!("│  2/5  Recent DEX Trades (XLM/USDC)                     │");
    println!("└─────────────────────────────────────────────────────────┘");
    if !budget.check(COST_PER_CALL) {
        println!("  ⛔ Budget exceeded!");
    } else {
        println!("  Budget: {} → spending ${:.4}...", budget.display(), COST_PER_CALL);
        match call_with_payment(&client, &format!("{}/dex-trades?limit=5", base), &config.agent_secret_key, &config.stellar_rpc_url).await {
            Ok(data) => {
                budget.spend(COST_PER_CALL);
                let count = data["count"].as_u64().unwrap_or(0);
                println!("      ✅ {} recent trades retrieved", count);
                if let Some(trades) = data["trades"].as_array() {
                    for t in trades.iter().take(3) {
                        println!("         {} XLM @ {} USDC ({})",
                            t["base_amount"].as_str().unwrap_or("?"),
                            t["price"].as_str().unwrap_or("?"),
                            &t["ledger_close_time"].as_str().unwrap_or("?")[..19]);
                    }
                }
            }
            Err(e) => println!("      ❌ {}", e),
        }
    }
    println!();
    pause(800).await;

    // ── Tool 3: Liquidity Pools ───────────────────────────────────────
    println!("┌─────────────────────────────────────────────────────────┐");
    println!("│  3/5  AMM Liquidity Pools                              │");
    println!("└─────────────────────────────────────────────────────────┘");
    if !budget.check(COST_PER_CALL) {
        println!("  ⛔ Budget exceeded!");
    } else {
        println!("  Budget: {} → spending ${:.4}...", budget.display(), COST_PER_CALL);
        match call_with_payment(&client, &format!("{}/liquidity-pools", base), &config.agent_secret_key, &config.stellar_rpc_url).await {
            Ok(data) => {
                budget.spend(COST_PER_CALL);
                let count = data["count"].as_u64().unwrap_or(0);
                println!("      ✅ {} liquidity pools found", count);
                if let Some(pools) = data["pools"].as_array() {
                    for p in pools.iter().take(3) {
                        let reserves: Vec<String> = p["reserves"].as_array().unwrap_or(&vec![]).iter()
                            .map(|r| format!("{} {}", r["amount"].as_str().unwrap_or("0"), r["asset"].as_str().unwrap_or("?")))
                            .collect();
                        println!("         Pool {} — {}", p["id"].as_str().unwrap_or("?"), reserves.join(" / "));
                    }
                }
            }
            Err(e) => println!("      ❌ {}", e),
        }
    }
    println!();
    pause(800).await;

    // ── Tool 4: Network Stats ─────────────────────────────────────────
    println!("┌─────────────────────────────────────────────────────────┐");
    println!("│  4/5  Stellar Network Stats                            │");
    println!("└─────────────────────────────────────────────────────────┘");
    if !budget.check(COST_PER_CALL) {
        println!("  ⛔ Budget exceeded!");
    } else {
        println!("  Budget: {} → spending ${:.4}...", budget.display(), COST_PER_CALL);
        match call_with_payment(&client, &format!("{}/network-stats", base), &config.agent_secret_key, &config.stellar_rpc_url).await {
            Ok(data) => {
                budget.spend(COST_PER_CALL);
                println!("      ✅ Ledger: #{}  Protocol: v{}  Base fee: {} stroops",
                    data["latest_ledger"].as_u64().unwrap_or(0),
                    data["protocol_version"].as_u64().unwrap_or(0),
                    data["base_fee_stroops"].as_str().unwrap_or("?"));
            }
            Err(e) => println!("      ❌ {}", e),
        }
    }
    println!();
    pause(800).await;

    // ── Tool 5: Account Analysis ──────────────────────────────────────
    println!("┌─────────────────────────────────────────────────────────┐");
    println!("│  5/5  Agent Account Analysis (Horizon)                 │");
    println!("└─────────────────────────────────────────────────────────┘");
    if !budget.check(COST_PER_CALL) {
        println!("  ⛔ Budget exceeded!");
    } else {
        println!("  Budget: {} → spending ${:.4}...", budget.display(), COST_PER_CALL);
        let url = format!("{}/account?address={}", base, config.agent_stellar_address);
        match call_with_payment(&client, &url, &config.agent_secret_key, &config.stellar_rpc_url).await {
            Ok(data) => {
                budget.spend(COST_PER_CALL);
                println!("      ✅ Account funded: {}", data["is_funded"]);
                if let Some(balances) = data["balances"].as_array() {
                    for b in balances {
                        println!("         {} → {}", b["asset"].as_str().unwrap_or("?"), b["balance"].as_str().unwrap_or("0"));
                    }
                }
            }
            Err(e) => println!("      ❌ {}", e),
        }
    }
    println!();

    pause(1000).await;

    // ── Session Summary ───────────────────────────────────────────────
    println!("━━━ Session Summary ━━━");
    println!();
    println!("  Budget used:  {}", budget.display());
    println!("  Remaining:    ${:.4} USDC", budget.remaining());
    println!();

    // ── On-Chain Verification ─────────────────────────────────────────
    println!("━━━ On-Chain Settlement Verification ━━━");
    println!();
    print!("  Waiting for finality");
    for _ in 0..5 { pause(1000).await; print!("."); }
    println!(" done");
    println!();

    let agent_after = get_usdc_balance(&config.agent_stellar_address).await?;
    let server_after = get_usdc_balance(&config.server_stellar_address).await?;
    let agent_spent = agent_before - agent_after;
    let server_earned = server_after - server_before;

    println!("  Agent  USDC: ${:.7} → ${:.7}  (spent: ${:.7})", agent_before, agent_after, agent_spent);
    println!("  Server USDC: ${:.7} → ${:.7} (earned: ${:.7})", server_before, server_after, server_earned);

    if agent_spent > 0.0 {
        println!();
        println!("  ✅ {} API calls, ${:.7} USDC settled on Stellar testnet", budget.calls, agent_spent);
        println!();
        println!("  🔗 Verify on Stellar Expert:");
        println!("     Agent:  https://stellar.expert/explorer/testnet/account/{}", config.agent_stellar_address);
        println!("     Server: https://stellar.expert/explorer/testnet/account/{}", config.server_stellar_address);
    }

    println!();
    println!("  Contracts on testnet:");
    println!("     ServiceRegistry: {}", config.service_registry_contract);
    println!("     SmartAccount:    {}", config.smart_account_contract);
    println!();
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║  Demo complete. {} API calls · {} x402 payments · all on-chain  ║", budget.calls, budget.calls);
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    Ok(())
}
