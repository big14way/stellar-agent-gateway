use axum::{extract::Query, Json};
use serde::{Deserialize, Serialize};
use tracing::warn;

const HORIZON: &str = "https://horizon-testnet.stellar.org";

#[derive(Deserialize)]
pub struct AccountQuery {
    pub address: String,
}

#[derive(Serialize)]
pub struct BalanceEntry {
    pub asset: String,
    pub balance: String,
}

#[derive(Serialize)]
pub struct AccountResponse {
    pub address: String,
    pub balances: Vec<BalanceEntry>,
    pub trustline_count: u32,
    pub subentry_count: u32,
    pub sequence: String,
    pub last_modified_ledger: u64,
    pub is_funded: bool,
    pub source: String,
}

async fn fetch_live_account(address: &str) -> Option<AccountResponse> {
    let url = format!("{}/accounts/{}", HORIZON, address);
    let resp = reqwest::get(&url).await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let data: serde_json::Value = resp.json().await.ok()?;

    let raw_balances = data["balances"].as_array()?;
    let mut balances = Vec::new();
    let mut trustline_count = 0u32;

    for b in raw_balances {
        let asset_type = b["asset_type"].as_str().unwrap_or("");
        let balance = b["balance"].as_str().unwrap_or("0").to_string();
        if asset_type == "native" {
            balances.push(BalanceEntry {
                asset: "XLM (native)".to_string(),
                balance,
            });
        } else {
            let code = b["asset_code"].as_str().unwrap_or("?");
            let issuer = b["asset_issuer"].as_str().unwrap_or("");
            let short_issuer = if issuer.len() > 8 {
                format!("{}...{}", &issuer[..4], &issuer[issuer.len()-4..])
            } else {
                issuer.to_string()
            };
            balances.push(BalanceEntry {
                asset: format!("{} ({})", code, short_issuer),
                balance,
            });
            trustline_count += 1;
        }
    }

    let subentry_count = data["subentry_count"].as_u64().unwrap_or(0) as u32;
    let sequence = data["sequence"].as_str().unwrap_or("0").to_string();
    let last_modified = data["last_modified_ledger"].as_u64().unwrap_or(0);

    Some(AccountResponse {
        address: address.to_string(),
        balances,
        trustline_count,
        subentry_count,
        sequence,
        last_modified_ledger: last_modified,
        is_funded: true,
        source: "Stellar Horizon (live)".to_string(),
    })
}

pub async fn analyze_account(Query(q): Query<AccountQuery>) -> Json<AccountResponse> {
    match fetch_live_account(&q.address).await {
        Some(account) => Json(account),
        None => {
            warn!("Failed to fetch account {}, returning unfunded response", q.address);
            Json(AccountResponse {
                address: q.address,
                balances: vec![],
                trustline_count: 0,
                subentry_count: 0,
                sequence: "0".to_string(),
                last_modified_ledger: 0,
                is_funded: false,
                source: "Stellar Horizon (account not found or unfunded)".to_string(),
            })
        }
    }
}
