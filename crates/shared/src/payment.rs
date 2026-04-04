// Real Soroban authorization entry signing for x402 payments on Stellar.
// Uses stellar-xdr =23.0.0 (matching soroban-sdk 23.5.3's dependency).
//
// Architecture A: agent's G-address signs directly.
// The `from` field in the USDC transfer is the agent's G-address.
// The agent's USDC balance is debited on-chain when the facilitator settles.

use anyhow::{anyhow, Context, Result};
use ed25519_dalek::{Signer, SigningKey};
use rand::Rng;
use sha2::{Digest, Sha256};
use stellar_strkey::{ed25519::PrivateKey as StrkeyPrivateKey, Strkey};
use stellar_xdr::curr::{
    AccountId, ContractId, Hash, HashIdPreimage, HashIdPreimageSorobanAuthorization,
    HostFunction, Int128Parts, InvokeContractArgs, InvokeHostFunctionOp,
    Limits, Memo, MuxedAccount, Operation, OperationBody, Preconditions,
    PublicKey, ScAddress, ScBytes, ScMap, ScMapEntry, ScSymbol, ScVal,
    ScVec, SequenceNumber, SorobanAddressCredentials, SorobanAuthorizationEntry,
    SorobanAuthorizedFunction, SorobanAuthorizedInvocation,
    SorobanCredentials, Transaction, TransactionEnvelope, TransactionExt,
    TransactionV1Envelope, Uint256, VecM, WriteXdr,
};

use crate::x402::PaymentRequirements;

// ── Internal helpers ──────────────────────────────────────────────────────

fn network_passphrase(network: &str) -> Result<&'static str> {
    match network {
        "stellar:testnet" => Ok("Test SDF Network ; September 2015"),
        "stellar:pubnet"  => Ok("Public Global Stellar Network ; September 2015"),
        other             => Err(anyhow!("Unsupported Stellar network: {}", other)),
    }
}

fn decode_sc_address(address: &str) -> Result<ScAddress> {
    let strkey = Strkey::from_string(address)
        .map_err(|e| anyhow!("Invalid Stellar address '{}': {:?}", address, e))?;
    match strkey {
        Strkey::PublicKeyEd25519(pk) => Ok(ScAddress::Account(AccountId(
            PublicKey::PublicKeyTypeEd25519(Uint256(pk.0)),
        ))),
        Strkey::Contract(c) => Ok(ScAddress::Contract(ContractId(Hash(c.0)))),
        _ => Err(anyhow!("Address type not supported for '{}'", address)),
    }
}

fn parse_i128_amount(s: &str) -> Result<Int128Parts> {
    let v: i128 = s.parse().with_context(|| format!("Invalid amount: '{}'", s))?;
    Ok(Int128Parts {
        hi: (v >> 64) as i64,
        lo: v as u64,
    })
}

// Encode ed25519 signature as AccountEd25519Signature ScVal per CAP-0046-11.
// Format: ScVal::Vec([ ScVal::Map([ {public_key: Bytes(32)}, {signature: Bytes(64)} ]) ])
// Keys MUST be alphabetically ordered: "public_key" before "signature".
fn encode_ed25519_signature(pubkey: &[u8; 32], sig: &[u8; 64]) -> Result<ScVal> {
    let map_entries = vec![
        ScMapEntry {
            key: ScVal::Symbol(ScSymbol("public_key".try_into()?)),
            val: ScVal::Bytes(ScBytes(pubkey.to_vec().try_into()?)),
        },
        ScMapEntry {
            key: ScVal::Symbol(ScSymbol("signature".try_into()?)),
            val: ScVal::Bytes(ScBytes(sig.to_vec().try_into()?)),
        },
    ];
    let sc_map = ScMap(map_entries.try_into()?);
    Ok(ScVal::Vec(Some(ScVec(
        vec![ScVal::Map(Some(sc_map))].try_into()?,
    ))))
}

// Fetch the current ledger sequence from Soroban RPC.
async fn get_latest_ledger(rpc_url: &str) -> Result<u32> {
    #[derive(serde::Deserialize)]
    struct RpcResponse { result: LedgerResult }
    #[derive(serde::Deserialize)]
    struct LedgerResult { sequence: u32 }

    let body = serde_json::json!({
        "jsonrpc": "2.0", "id": 1,
        "method": "getLatestLedger", "params": {}
    });
    let resp = reqwest::Client::new()
        .post(rpc_url)
        .json(&body)
        .send()
        .await
        .context("getLatestLedger: request failed")?
        .json::<RpcResponse>()
        .await
        .context("getLatestLedger: failed to parse response")?;
    Ok(resp.result.sequence)
}

// ── Public API ────────────────────────────────────────────────────────────

/// Build and sign an x402 Stellar payment.
///
/// Signs a SorobanAuthorizationEntry for transfer(from, to, amount) on the
/// USDC SEP-41 contract using the agent's ed25519 secret key.
///
/// Returns base64-encoded TransactionEnvelope XDR. The x402 facilitator
/// verifies the auth entry, wraps the transaction in a fee-bump, and submits.
pub async fn sign_x402_payment(
    secret_key: &str,
    requirements: &PaymentRequirements,
    rpc_url: &str,
) -> Result<String> {
    // A: Decode agent secret key
    let strkey_sk = StrkeyPrivateKey::from_string(secret_key)
        .map_err(|e| anyhow!("Bad secret key: {:?}", e))?;
    let signing_key = SigningKey::from_bytes(&strkey_sk.0);
    let pubkey_bytes: [u8; 32] = signing_key.verifying_key().to_bytes();

    // B: Compute network_id = SHA256(passphrase)
    let passphrase = network_passphrase(&requirements.network)?;
    let network_id: [u8; 32] = Sha256::digest(passphrase.as_bytes()).into();

    // C: Build ScAddress values
    let from_sc_address = ScAddress::Account(AccountId(
        PublicKey::PublicKeyTypeEd25519(Uint256(pubkey_bytes)),
    ));
    let to_sc_address = decode_sc_address(&requirements.pay_to)
        .context("Invalid pay_to address")?;
    let token_sc_address = decode_sc_address(&requirements.asset)
        .context("Invalid asset address")?;

    // D: Parse payment amount (USDC has 7 decimals; "10000" = $0.001)
    let amount_parts = parse_i128_amount(&requirements.amount)?;

    // E: Get current ledger, set expiry within facilitator's allowed window.
    //    Facilitator allows: ceil(maxTimeoutSeconds / 5) ledgers from current.
    //    maxTimeoutSeconds=60 → max 12 ledgers. Use 10 for safety margin.
    let latest_ledger = get_latest_ledger(rpc_url).await?;
    let signature_expiration_ledger: u32 = latest_ledger + 10;

    // F: Random nonce for replay protection
    let nonce: i64 = rand::thread_rng().gen_range(0..i64::MAX);

    // G: Build the authorized function for transfer(from, to, amount)
    let authorized_fn = SorobanAuthorizedFunction::ContractFn(
        InvokeContractArgs {
            contract_address: token_sc_address.clone(),
            function_name: ScSymbol("transfer".try_into()?),
            args: vec![
                    ScVal::Address(from_sc_address.clone()),
                    ScVal::Address(to_sc_address),
                    ScVal::I128(amount_parts.clone()),
                ]
                .try_into()?,
        },
    );

    // H: Build the invocation tree (single root, no sub-invocations)
    let root_invocation = SorobanAuthorizedInvocation {
        function: authorized_fn,
        sub_invocations: VecM::default(),
    };

    // I: Build and sign the preimage
    //    preimage = XDR(HashIdPreimage::SorobanAuthorization(...))
    //    hash    = SHA256(preimage_xdr)
    //    sig     = ed25519_sign(hash)
    let preimage = HashIdPreimage::SorobanAuthorization(
        HashIdPreimageSorobanAuthorization {
            network_id: Hash(network_id),
            nonce,
            signature_expiration_ledger,
            invocation: root_invocation.clone(),
        },
    );
    let preimage_xdr = preimage
        .to_xdr(Limits::none())
        .context("Failed to XDR-encode signing preimage")?;
    let payload_hash: [u8; 32] = Sha256::digest(&preimage_xdr).into();
    let signature = signing_key.sign(&payload_hash);
    let sig_bytes: [u8; 64] = signature.to_bytes();

    // J: Encode the signature as AccountEd25519Signature ScVal
    let sig_scval = encode_ed25519_signature(&pubkey_bytes, &sig_bytes)?;

    // K: Build the SorobanAuthorizationEntry with Address credentials
    let auth_entry = SorobanAuthorizationEntry {
        credentials: SorobanCredentials::Address(SorobanAddressCredentials {
            address: from_sc_address.clone(),
            nonce,
            signature_expiration_ledger,
            signature: sig_scval,
        }),
        root_invocation,
    };

    // L: Build the InvokeHostFunction operation
    //    This calls transfer() on the USDC SAC
    let invoke_args = InvokeContractArgs {
        contract_address: token_sc_address,
        function_name: ScSymbol("transfer".try_into()?),
        args: vec![
                ScVal::Address(from_sc_address),
                ScVal::Address(decode_sc_address(&requirements.pay_to)?),
                ScVal::I128(amount_parts),
            ]
            .try_into()?,
    };
    let host_fn_op = InvokeHostFunctionOp {
        host_function: HostFunction::InvokeContract(invoke_args),
        auth: vec![auth_entry].try_into()?,
    };

    // M: Build the Transaction
    //    fee and seq_num are placeholders — the facilitator fee-bumps.
    //    Facilitator validates fee >= minimum (10M stroops for Soroban).
    let tx = Transaction {
        source_account: MuxedAccount::Ed25519(Uint256(pubkey_bytes)),
        fee: 1_000_000,
        seq_num: SequenceNumber(0),
        cond: Preconditions::None,
        memo: Memo::None,
        operations: vec![Operation {
            source_account: None,
            body: OperationBody::InvokeHostFunction(host_fn_op),
        }]
        .try_into()?,
        ext: TransactionExt::V0,
    };

    // N: Wrap in envelope (no envelope signature — agent only signs the auth entry)
    let envelope = TransactionEnvelope::Tx(TransactionV1Envelope {
        tx,
        signatures: VecM::default(),
    });

    // O: Return base64 XDR
    use base64::Engine;
    let xdr_bytes = envelope
        .to_xdr(Limits::none())
        .context("Failed to encode TransactionEnvelope to XDR")?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&xdr_bytes))
}
