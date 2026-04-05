# Stellar Agent Gateway

An x402-powered agent service marketplace on Stellar where AI agents autonomously pay for API tools using real USDC micropayments via Soroban authorization entry signing. When an agent calls a tool, the MCP server intercepts the HTTP 402 response, cryptographically signs a Soroban auth entry for the USDC transfer, and retries with the payment header — all without human intervention. The OZ Facilitator verifies and settles the payment on-chain. Built for the Stellar Hacks: Agents hackathon (April 2026).

## Live Demo

**Dashboard:** [stellar-agent-gateway.vercel.app](https://stellar-agent-gateway.vercel.app)
Shows live agent/server USDC balances, registered services, transaction feed, and x402 payment flow diagram. Account data updates in real-time from Stellar Horizon — no wallet extension needed.

**On-chain contracts (Stellar Testnet):**
- ServiceRegistry: [`CAJZB47S...`](https://stellar.expert/explorer/testnet/contract/CAJZB47SSVIORFB4J7AHFVSA2HOSQU2YX4KJ5DORL4KK5EUX4AUSQJ4Y) — 3 services registered
- SmartAccount: [`CCRXBYAV...`](https://stellar.expert/explorer/testnet/contract/CCRXBYAVZBCBYLYCZCJPLO22LYAIZGLR4QCJNFCHOZL2WAQJFWVGV7R3) — OZ policy-based spending

**Agent account:** [`GCLLO5UE...`](https://stellar.expert/explorer/testnet/account/GCLLO5UEXQ5KS3WSSBFTZ4S3TJMJHKDCDIMU7WUIECSKDZJMEDX22AE7) — view x402 payment transactions on Stellar Expert

### Run the E2E Demo Locally

The full payment flow (sign → pay → settle → data) runs locally. The dashboard on Vercel updates in real-time as payments settle.

```bash
# Terminal 1: Start x402 services
make dev-services

# Terminal 2: Run the demo (3 paid API calls, real USDC on-chain)
cargo run -p e2e-test
```

The demo shows an AI agent performing a portfolio analysis — calling 3 live APIs (Stellar DEX price, Horizon account data, weather), paying $0.001 USDC per call, and synthesizing the results. Watch the dashboard balances update as payments settle.

## Architecture

```
  Claude (Cursor / Claude Desktop)
    │ MCP JSON-RPC over stdio
    ▼
  stellar-mcp-server (Rust/rmcp)
    │ GET /weather?city=Lagos
    ▼
  x402-services (Rust/Axum)   ──→  HTTP 402 PAYMENT-REQUIRED
    │                                        │
    │ PAYMENT-SIGNATURE                      │ (signed Soroban auth entry)
    │ (tx XDR with signed auth entry)        │
    ▼                                        │
  OZ Facilitator                    ◀────────┘
    │ fee-bump transaction
    ▼
  Stellar Testnet (USDC SAC)
    │ settled payment
    ▼
  x402-services   ──→  HTTP 200 + JSON response
    │
    ▼
  Claude Desktop ← response data
    │
    ▼
  Next.js Dashboard  ← Horizon API (live tx feed)
```

## Components

| Component | Language | Description |
|-----------|----------|-------------|
| **ServiceRegistry** | Soroban (Rust) | On-chain service discovery. Stores service name, URL, price, owner, and tags |
| **SmartAccount** | Soroban (Rust) | OpenZeppelin policy-based spending limits using `stellar-accounts` v0.6.0 |
| **x402-services** | Rust (Axum) | 3 API routes gated by x402. Live data from Stellar DEX, Horizon, and wttr.in |
| **stellar-mcp-server** | Rust (rmcp) | 4 MCP tools for Claude. Real x402 signing with ed25519 over Soroban auth entries |
| **e2e-test** | Rust | Demo binary: multi-step agent scenario with on-chain settlement verification |
| **Dashboard** | Next.js 16 / React 19 | Live balances, tx feed, service catalog. Deployed on Vercel |

## Prerequisites

- **Rust 1.80+** with `wasm32v1-none` target (`rustup target add wasm32v1-none`)
- **Node.js 22+**
- **Stellar CLI 25.x** (`stellar --version`)
- **OZ Facilitator API key** from https://channels.openzeppelin.com/testnet/gen
- **Test USDC** from https://faucet.circle.com (Stellar Testnet)

## Quick Start

### 1. Generate and fund testnet accounts

```bash
stellar keys generate server --network testnet --fund
stellar keys generate agent  --network testnet --fund
```

### 2. Configure environment

```bash
cp .env.example .env
# Edit .env with your keys:
#   stellar keys show server    → SERVER_SECRET_KEY
#   stellar keys address server → SERVER_STELLAR_ADDRESS
#   stellar keys show agent     → AGENT_SECRET_KEY
#   stellar keys address agent  → AGENT_STELLAR_ADDRESS
#   Paste your OZ API key       → FACILITATOR_API_KEY
```

### 3. Build and deploy contracts

```bash
make build
bash scripts/deploy-contracts.sh
```

### 4. Set up agent account (USDC trustline)

```bash
bash scripts/setup-agent.sh
# Then visit https://faucet.circle.com → Stellar Testnet
# Fund both AGENT and SERVER addresses with test USDC
```

### 5. Register services on-chain

```bash
bash scripts/register-services.sh
```

### 6. Run the full demo

```bash
# Terminal 1: Start x402 services
make dev-services

# Terminal 2: Run end-to-end payment demo
cargo run -p e2e-test

# Optional: Dashboard (also live at stellar-agent-gateway.vercel.app)
cd dashboard && npm install && npm run dev
```

### 7. Connect to Claude (optional)

**Cursor:** MCP config is at `.cursor/mcp.json` — set env vars and restart.

**Claude Desktop:** See [docs/add-to-claude-desktop.md](docs/add-to-claude-desktop.md).

## x402 Payment Flow

1. **Agent calls API** (e.g., `GET /weather?city=Lagos`)
2. **Server returns HTTP 402** with `PAYMENT-REQUIRED` header (amount, USDC contract, pay-to address)
3. **Agent signs Soroban auth entry** — ed25519 over `transfer(agent, server, amount)` on USDC SAC
4. **Agent retries** with `PAYMENT-SIGNATURE` header containing signed transaction XDR
5. **Server forwards to OZ Facilitator** for verification
6. **Facilitator fee-bumps and submits** to Stellar Testnet — real USDC transfer settles on-chain
7. **Server returns HTTP 200** with live data (Stellar DEX price, Horizon account data, weather)

## Smart Contract Details

### ServiceRegistry ([`CAJZB47S...`](https://stellar.expert/explorer/testnet/contract/CAJZB47SSVIORFB4J7AHFVSA2HOSQU2YX4KJ5DORL4KK5EUX4AUSQJ4Y))

On-chain service catalog with 8 functions:

- `register_service(owner, name, url, description, price_usdc, tags)` → service ID
- `get_service(id)` → `ServiceEntry`
- `get_all_services()` → active services
- `get_services_by_owner(owner)` → owner's services
- `deregister_service(id)` — sets `active = false`
- `update_price(id, new_price)` — owner-only
- `get_count()` → total registered

### SmartAccount ([`CCRXBYAV...`](https://stellar.expert/explorer/testnet/contract/CCRXBYAVZBCBYLYCZCJPLO22LYAIZGLR4QCJNFCHOZL2WAQJFWVGV7R3))

OpenZeppelin `stellar-accounts` v0.6.0 smart wallet with 15 functions:

- Full `SmartAccount` trait (11 methods): context rules, signers, policies
- `grant_agent_usdc_access(agent_address, usdc_contract, valid_until_ledger)`
- `CustomAccountInterface::__check_auth` — delegates to `do_check_auth`
- Uses `Signer::Delegated(Address)` for agent G-address authorization
- `ContextRuleType::CallContract(usdc_address)` scopes auth to USDC contract only

## MCP Tools

| Tool | Price | Parameters | Description |
|------|-------|------------|-------------|
| `get_weather` | $0.001 USDC | `city: string` | Live weather data from wttr.in |
| `get_xlm_price` | $0.001 USDC | *(none)* | Live XLM/USDC rate from Stellar DEX orderbook |
| `analyze_account` | $0.001 USDC | `address: string` | Live account data from Stellar Horizon |
| `list_services` | FREE | *(none)* | Lists all available services with prices |

## Known Limitations

- **Testnet only** — All USDC payments use testnet tokens with no real monetary value. Switching to pubnet requires changing the network passphrase and using real USDC.
- **Local x402 server** — The payment-gated API server runs locally. The Vercel dashboard shows live on-chain data but cannot trigger payments independently.
- **Dashboard budget meter** — The UI shows wallet balances but does not yet read spending limits from the SmartAccount contract on-chain.

## Links

- [Stellar x402 docs](https://developers.stellar.org/docs/build/apps/x402)
- [OpenZeppelin Smart Accounts](https://docs.openzeppelin.com/stellar-contracts/accounts/smart-account)
- [x402 protocol spec](https://github.com/coinbase/x402)
- [rmcp — Rust MCP SDK](https://github.com/modelcontextprotocol/rust-sdk)
- [OZ x402 Facilitator](https://github.com/OpenZeppelin/relayer-plugin-x402-facilitator)
- [Stellar Hacks: Agents Hackathon](https://dorahacks.io/hackathon/stellar-agents-x402-stripe-mpp/detail)
