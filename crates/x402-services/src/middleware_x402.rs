use axum::{
    body::Body,
    extract::State,
    middleware::Next,
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
};
use shared::{
    AppConfig,
    x402::{
        encode_header, decode_header,
        PaymentPayload, PaymentRequired, PaymentRequirements, Resource,
    },
};
use tracing::{error, info, warn};

fn build_402_response(config: &AppConfig, resource_url: &str, description: &str) -> Response {
    let requirements = PaymentRequired {
        x402_version: 2,
        error: "Payment required".to_string(),
        resource: Resource {
            url: resource_url.to_string(),
            description: description.to_string(),
            mime_type: "application/json".to_string(),
        },
        accepts: vec![PaymentRequirements {
            scheme: "exact".to_string(),
            network: "stellar:testnet".to_string(),
            amount: "10000".to_string(),      // $0.001 USDC (7 decimals)
            asset: config.usdc_contract.clone(),
            pay_to: config.server_stellar_address.clone(),
            max_timeout_seconds: 60,
            extra: serde_json::json!({ "areFeesSponsored": true }),
        }],
        extensions: serde_json::Value::Object(serde_json::Map::new()),
    };

    let header_value = encode_header(&requirements);
    let body_json = serde_json::to_string(&requirements).unwrap_or_default();

    Response::builder()
        .status(StatusCode::PAYMENT_REQUIRED)
        .header("content-type", "application/json")
        .header("PAYMENT-REQUIRED", header_value)
        .body(Body::from(body_json))
        .unwrap()
}

pub async fn x402_payment_guard(
    State(config): State<AppConfig>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let resource_url = format!("http://localhost:{}{}", config.port, req.uri().path());

    // Check for payment header (accept both PAYMENT-SIGNATURE and legacy X-PAYMENT)
    let payment_val = req.headers()
        .get("PAYMENT-SIGNATURE")
        .or_else(|| req.headers().get("X-PAYMENT"))
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    let header_value = match payment_val {
        Some(v) => v,
        None => return build_402_response(&config, &resource_url, "No payment header"),
    };

    // Decode payment payload
    let payload: PaymentPayload = match decode_header(&header_value) {
        Ok(p) => p,
        Err(e) => {
            warn!("Bad PAYMENT-SIGNATURE header: {}", e);
            return build_402_response(&config, &resource_url, "Malformed payment");
        }
    };

    // Verify with OZ facilitator
    let requirements = PaymentRequirements {
        scheme: "exact".to_string(),
        network: "stellar:testnet".to_string(),
        amount: "10000".to_string(),
        asset: config.usdc_contract.clone(),
        pay_to: config.server_stellar_address.clone(),
        max_timeout_seconds: 60,
        extra: serde_json::json!({ "areFeesSponsored": true }),
    };

    let verify_body = serde_json::json!({
        "paymentPayload": payload,
        "paymentRequirements": requirements,
    });

    let verify_resp = match reqwest::Client::new()
        .post(format!("{}/verify", config.facilitator_url))
        .header("Authorization", format!("Bearer {}", config.facilitator_api_key))
        .json(&verify_body)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            error!("Facilitator verify failed: {}", e);
            return (StatusCode::SERVICE_UNAVAILABLE, "Facilitator unavailable").into_response();
        }
    };

    let verify_json: serde_json::Value = match verify_resp.json().await {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to parse verify response: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Bad verify response").into_response();
        }
    };

    let is_valid = verify_json
        .get("isValid")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !is_valid {
        let reason = verify_json
            .get("invalidReason")
            .and_then(|r| r.as_str())
            .unwrap_or("Payment invalid");
        warn!("Payment rejected: {}", reason);
        return build_402_response(&config, &resource_url, reason);
    }

    let payer = verify_json
        .get("payer")
        .and_then(|p| p.as_str())
        .unwrap_or("unknown");
    info!("Payment verified from: {}", payer);

    // Serve the actual response
    let response = next.run(req).await;

    // Settle on-chain in background (fire-and-forget)
    let settle_url = format!("{}/settle", config.facilitator_url);
    let settle_key = config.facilitator_api_key.clone();
    let settle_body = serde_json::json!({
        "paymentPayload": payload,
        "paymentRequirements": requirements,
    });
    tokio::spawn(async move {
        match reqwest::Client::new()
            .post(&settle_url)
            .header("Authorization", format!("Bearer {}", settle_key))
            .json(&settle_body)
            .send()
            .await
        {
            Ok(r) => {
                if let Ok(j) = r.json::<serde_json::Value>().await {
                    let tx = j.get("transaction").and_then(|t| t.as_str()).unwrap_or("?");
                    info!("Payment settled. tx_hash: {}", tx);
                }
            }
            Err(e) => error!("Settlement error: {}", e),
        }
    });

    response
}
