use axum::{extract::Query, Json};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AccountQuery { pub address: String }

#[derive(Serialize)]
pub struct AccountResponse {
    pub address: String,
    pub xlm_balance: String,
    pub usdc_balance: String,
    pub trustline_count: u32,
    pub transaction_count: u32,
    pub account_age_days: u32,
    pub is_active: bool,
}

pub async fn analyze_account(Query(q): Query<AccountQuery>) -> Json<AccountResponse> {
    Json(AccountResponse {
        address: q.address,
        xlm_balance: "1250.50".to_string(),
        usdc_balance: "89.23".to_string(),
        trustline_count: 3,
        transaction_count: 127,
        account_age_days: 412,
        is_active: true,
    })
}
