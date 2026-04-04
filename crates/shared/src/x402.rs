use serde::{Deserialize, Serialize};

// All x402 V2 protocol types. camelCase matches the JSON wire format.

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    pub url: String,
    pub description: String,
    pub mime_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentRequirements {
    pub scheme: String,
    pub network: String,
    pub amount: String,            // token base units as string — USDC has 7 decimals
    pub asset: String,             // USDC SEP-41 contract address (C...)
    pub pay_to: String,            // recipient Stellar address (G...)
    pub max_timeout_seconds: u64,
    pub extra: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentRequired {
    pub x402_version: u32,
    pub error: String,
    pub resource: Resource,
    pub accepts: Vec<PaymentRequirements>,
    pub extensions: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarPayload {
    pub transaction: String,       // base64-encoded TransactionEnvelope XDR
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentPayload {
    pub x402_version: u32,
    pub accepted: PaymentRequirements,
    pub payload: StellarPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyRequest {
    pub payment_payload: PaymentPayload,
    pub payment_requirements: PaymentRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyResponse {
    pub is_valid: bool,
    pub invalid_reason: Option<String>,
    pub payer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettleResponse {
    pub success: bool,
    pub transaction: String,
    pub network: String,
    pub payer: String,
}

// Encode any serializable value to base64(JSON) for HTTP headers
pub fn encode_header<T: Serialize>(value: &T) -> String {
    use base64::Engine;
    let json = serde_json::to_string(value).expect("encode_header: serialization failed");
    base64::engine::general_purpose::STANDARD.encode(json.as_bytes())
}

// Decode a base64(JSON) header value into any deserializable type
pub fn decode_header<T: for<'de> Deserialize<'de>>(header_value: &str) -> anyhow::Result<T> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(header_value.trim())
        .map_err(|e| anyhow::anyhow!("base64 decode error: {}", e))?;
    serde_json::from_slice(&bytes).map_err(|e| anyhow::anyhow!("JSON decode error: {}", e))
}
