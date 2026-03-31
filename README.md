# Stellar Agent Gateway

An x402-powered agent service marketplace on Stellar where Claude autonomously pays for API tools using real USDC micropayments via Soroban authorization entry signing. When an AI agent calls a tool, the MCP server intercepts the HTTP 402 response, cryptographically signs a Soroban auth entry for the USDC transfer, and retries with the payment header — all without human intervention. The OZ Facilitator verifies and settles the payment on-chain. Built for the Stellar Hacks: Agents hackathon (April 2026).

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
| **ServiceRegistry** | Soroban (Rust) | On-chain service discovery. Stores service name, URL, price, owner, and tags. Deployed at `CAJZB47SSVIORFB4J7AHFVSA2HOSQU2YX4KJ5DORL4KK5EUX4AUSQJ4Y` |
| **SmartAccount** | Soroban (Rust) | OpenZeppelin policy-based spending limits using `stellar-accounts` v0.6.0. Supports `Signer::Delegated` for agent G-address auth over USDC transfers. Deployed at `CCRXBYAVZBCBYLYCZCJPLO22LYAIZGLR4QCJNFCHOZL2WAQJFWVGV7R3` |
| **x402-services** | Rust (Axum) | HTTP server with 3 API routes gated by x402 micropayments. Returns 402 with payment requirements, verifies via OZ Facilitator, settles on-chain in background |
| **stellar-mcp-server** | Rust (rmcp) | MCP server exposing 4 tools to Claude. Handles the full x402 flow: request → 402 → sign Soroban auth entry → retry with payment header |
| **Dashboard** | Next.js 15 / React 19 | Service catalog, Freighter wallet connect, live XLM/USDC balances, transaction feed from Horizon |

## Prerequisites

- **Rust 1.80+** with `wasm32v1-none` target (`rustup target add wasm32v1-none`)
- **Node.js 22+**
- **Stellar CLI 25.x** (`stellar --version`)
- **Freighter wallet** browser extension (for dashboard)
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
# Then visit https://faucet.circle.com → Stellar Testnet → paste AGENT address
# Also fund the SERVER address with test USDC
```

### 5. Register services on-chain

```bash
bash scripts/register-services.sh
```

### 6. Start the x402 services server

```bash
make dev-services
# Verify: curl http://localhost:3001/health
# Paid routes return 402: curl http://localhost:3001/weather?city=Lagos
```

### 7. Run integration tests

```bash
bash scripts/test-integration.sh
```

### 8. Connect to Claude

**Cursor:** MCP config is at `.cursor/mcp.json` — set env vars and restart.

**Claude Desktop:** See [docs/add-to-claude-desktop.md](docs/add-to-claude-desktop.md).

### 9. Launch the dashboard

```bash
cd dashboard && npm install && npm run dev
# Open http://localhost:3000
```

## x402 Payment Flow

1. **Claude calls a tool** (e.g., `get_weather(city: "Lagos")`) via MCP JSON-RPC
2. **MCP server** sends `GET /weather?city=Lagos` to x402-services
3. **x402-services** returns **HTTP 402** with `PAYMENT-REQUIRED` header containing base64-encoded payment requirements (amount, USDC contract, pay-to address)
4. **MCP server** decodes the requirements and calls `sign_x402_payment()`
5. **Payment signer** builds a `SorobanAuthorizationEntry` for `transfer(agent, server, amount)` on the USDC SAC, signs it with the agent's ed25519 key, and wraps it in a `TransactionEnvelope`
6. **MCP server** retries the request with `PAYMENT-SIGNATURE` header containing the base64-encoded signed transaction XDR
7. **x402-services middleware** sends the payment to the **OZ Facilitator** for verification
8. **Facilitator** verifies the auth entry, fee-bumps the transaction, and submits it to **Stellar Testnet** — USDC is transferred from agent to server on-chain
9. **x402-services** returns **HTTP 200** with the actual data, which flows back to Claude

## Smart Contract Details

### ServiceRegistry (`CAJZB47SSVIORFB4J7AHFVSA2HOSQU2YX4KJ5DORL4KK5EUX4AUSQJ4Y`)

On-chain service catalog with 8 functions:

- `register_service(owner, name, url, description, price_usdc, tags)` → service ID
- `get_service(id)` → `ServiceEntry`
- `get_all_services()` → active services
- `get_services_by_owner(owner)` → owner's services
- `deregister_service(id)` — sets `active = false`
- `update_price(id, new_price)` — owner-only
- `get_count()` → total registered

[View on Stellar Expert](https://stellar.expert/explorer/testnet/contract/CAJZB47SSVIORFB4J7AHFVSA2HOSQU2YX4KJ5DORL4KK5EUX4AUSQJ4Y)

### SmartAccount (`CCRXBYAVZBCBYLYCZCJPLO22LYAIZGLR4QCJNFCHOZL2WAQJFWVGV7R3`)

OpenZeppelin `stellar-accounts` v0.6.0 smart wallet with 15 functions:

- Full `SmartAccount` trait (11 methods): context rules, signers, policies
- `grant_agent_usdc_access(agent_address, usdc_contract, valid_until_ledger)` — convenience method
- `CustomAccountInterface::__check_auth` — delegates to `do_check_auth`
- Uses `Signer::Delegated(Address)` for agent G-address authorization
- `ContextRuleType::CallContract(usdc_address)` scopes auth to USDC contract only

[View on Stellar Expert](https://stellar.expert/explorer/testnet/contract/CCRXBYAVZBCBYLYCZCJPLO22LYAIZGLR4QCJNFCHOZL2WAQJFWVGV7R3)

## MCP Tools

| Tool | Price | Parameters | Description |
|------|-------|------------|-------------|
| `get_weather` | $0.001 USDC | `city: string` — city name (e.g., "Lagos", "Tokyo") | Current weather data including temperature, humidity, wind, UV index |
| `get_xlm_price` | $0.001 USDC | *(none)* | XLM/USDC exchange rate with 24h change, volume, high/low |
| `analyze_account` | $0.002 USDC | `address: string` — Stellar G-address | Account balances (XLM, USDC), trustlines, transaction count, age |
| `list_services` | FREE | *(none)* | Lists all available services with names, prices, and descriptions |

## Known Limitations

- **Mock data responses** — Weather, price, and account endpoints return static values. In production these would call real APIs (OpenWeatherMap, DEX aggregator, Horizon).
- **Testnet only** — All USDC payments use testnet tokens with no real monetary value. Switching to pubnet requires changing the network passphrase and using real USDC.
- **Dashboard budget meter** — The UI shows wallet balances but does not yet read spending limits from the SmartAccount contract. This is a post-hackathon enhancement.
- **No persistent payment history** — The MCP server logs payments to stderr but does not store them. The dashboard reads transactions from Horizon which shows all account activity, not just x402 payments.

## Links

- [Stellar x402 docs](https://developers.stellar.org/docs/build/apps/x402)
- [OpenZeppelin Smart Accounts](https://docs.openzeppelin.com/stellar-contracts/accounts/smart-account)
- [x402 protocol spec](https://github.com/coinbase/x402)
- [rmcp — Rust MCP SDK](https://github.com/modelcontextprotocol/rust-sdk)
- [OZ x402 Facilitator](https://github.com/OpenZeppelin/relayer-plugin-x402-facilitator)
- [Stellar Hacks: Agents Hackathon](https://dorahacks.io/hackathon/stellar-agents-x402-stripe-mpp/detail)
