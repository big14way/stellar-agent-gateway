# Stellar Agent Gateway

> AI agents that can reason, plan, and act — but can't pay. We fixed that.

An x402-powered marketplace on Stellar where AI agents autonomously discover, pay for, and consume API services using real USDC micropayments. No API keys. No subscriptions. No human in the loop.

Built for [Stellar Hacks: Agents](https://dorahacks.io/hackathon/stellar-agents-x402-stripe-mpp/detail) (April 2026).

---

## 🎯 The Problem

Today's AI agents hit a wall the moment they need paid data. Every API requires keys, billing accounts, credit cards, and human setup — for every single service. An agent that needs weather data, market prices, and blockchain analytics must wait for a human to provision three separate accounts.

**This breaks agent autonomy.** The bottleneck killing the agent economy isn't intelligence — it's access.

## 💡 Our Solution

Stellar Agent Gateway turns HTTP into a payment rail using the [x402 protocol](https://www.x402.org/). An agent requests a resource, gets a price via HTTP 402, signs a Soroban authorization entry, and gets the data — in one round trip.

- **6 paid API endpoints** serving live data from Stellar DEX, Horizon, and wttr.in
- **7 MCP tools** for Claude/Cursor with real Soroban auth entry signing
- **2 Soroban smart contracts** — on-chain service registry + OZ SmartAccount spending policies
- **Session budget enforcement** — agents operate within defined spending limits
- **Real USDC settling on Stellar testnet** — verifiable on [Stellar Expert](https://stellar.expert/explorer/testnet/account/GCLLO5UEXQ5KS3WSSBFTZ4S3TJMJHKDCDIMU7WUIECSKDZJMEDX22AE7)

## 🚀 Live Demo

**Dashboard:** [stellar-agent-gateway.vercel.app](https://stellar-agent-gateway.vercel.app)
Live agent/server USDC balances, 6 service cards, transaction feed — updates every 5 seconds from Horizon.

**Run the E2E demo locally (recommended):**

```bash
# Terminal 1: Start x402 services
make dev-services

# Terminal 2: Run the demo — 5 paid API calls, real USDC on-chain
cargo run -p e2e-test
```

The demo simulates an AI agent performing a comprehensive Stellar market analysis — calling 5 live APIs with a $0.01 session budget, paying $0.001 per call, and producing a synthesized report. Watch the dashboard update as payments settle.

**On-chain (Stellar Testnet):**

| Resource | Link |
|----------|------|
| ServiceRegistry contract | [`CAJZB47S...`](https://stellar.expert/explorer/testnet/contract/CAJZB47SSVIORFB4J7AHFVSA2HOSQU2YX4KJ5DORL4KK5EUX4AUSQJ4Y) |
| SmartAccount contract | [`CCRXBYAV...`](https://stellar.expert/explorer/testnet/contract/CCRXBYAVZBCBYLYCZCJPLO22LYAIZGLR4QCJNFCHOZL2WAQJFWVGV7R3) |
| Agent account (see payments) | [`GCLLO5UE...`](https://stellar.expert/explorer/testnet/account/GCLLO5UEXQ5KS3WSSBFTZ4S3TJMJHKDCDIMU7WUIECSKDZJMEDX22AE7) |
| Server account (receives) | [`GAQQQN2Y...`](https://stellar.expert/explorer/testnet/account/GAQQQN2Y653HGR3U4SY4EMUBYROG3JVDBHZRSGTAOD3VM7EFFFCVBW6A) |

## 🏗️ Architecture

```
  AI Agent (Claude / Cursor / any MCP client)
    │
    │ MCP JSON-RPC (stdio)
    ▼
  ┌────────────────────────────────────┐
  │  stellar-mcp-server (Rust/rmcp)   │  7 tools, real Soroban signing
  │  Session budget: $0.01 max        │
  └──────────────┬─────────────────────┘
                 │ GET /price (no payment header)
                 ▼
  ┌────────────────────────────────────┐
  │  x402-services (Rust/Axum)        │  6 payment-gated endpoints
  │  Returns HTTP 402 + requirements  │  Live data: DEX, Horizon, wttr.in
  └──────────────┬─────────────────────┘
                 │ PAYMENT-SIGNATURE header
                 │ (base64 TransactionEnvelope XDR
                 │  with signed SorobanAuthorizationEntry)
                 ▼
  ┌────────────────────────────────────┐
  │  OZ Facilitator                   │  Verifies auth entry
  │  channels.openzeppelin.com        │  Fee-bumps transaction
  └──────────────┬─────────────────────┘
                 │ Submit to network
                 ▼
  ┌────────────────────────────────────┐
  │  Stellar Testnet                  │  USDC transfer settles (~5 sec)
  │  USDC SAC contract                │  Agent → Server
  └──────────────┬─────────────────────┘
                 │
                 ▼
  ┌────────────────────────────────────┐
  │  x402-services → HTTP 200         │  Returns live data
  │  MCP server → Agent → User        │
  └────────────────────────────────────┘
                 │
                 ▼
  ┌────────────────────────────────────┐
  │  Next.js Dashboard (Vercel)       │  Live balances + tx feed
  │  Horizon API polling (5s)         │  from Stellar Horizon
  └────────────────────────────────────┘
```

## 🔧 Components

| Component | Language | What It Does |
|-----------|----------|-------------|
| **ServiceRegistry** | Soroban (Rust) | On-chain service catalog — name, URL, price, owner, tags. 8 functions. |
| **SmartAccount** | Soroban (Rust) | OZ `stellar-accounts` v0.6.0 smart wallet. Spending policies, delegated signers, context rules. 15 functions. |
| **x402-services** | Rust (Axum) | 6 payment-gated API routes. x402 middleware verifies via OZ Facilitator, settles in background. All responses use live data. |
| **stellar-mcp-server** | Rust (rmcp 0.15) | 7 MCP tools for Claude/Cursor. Real ed25519 signing over Soroban auth entries. Session budget tracking. |
| **e2e-test** | Rust | Demo binary — 5-step market analysis scenario with budget enforcement and on-chain settlement verification. |
| **Dashboard** | Next.js 16 / React 19 | Live agent/server balances, tx feed, service catalog, x402 flow diagram. Deployed on Vercel. |

## 📊 MCP Tools

| Tool | Price | Parameters | Data Source |
|------|-------|------------|-------------|
| `get_xlm_price` | $0.001 USDC | — | Stellar DEX orderbook (Horizon) |
| `get_dex_trades` | $0.001 USDC | `limit: number` | Stellar DEX `/trades` (Horizon) |
| `get_liquidity_pools` | $0.001 USDC | — | Stellar AMM `/liquidity_pools` (Horizon) |
| `get_network_stats` | $0.001 USDC | — | Stellar `/fee_stats` + `/ledgers` (Horizon) |
| `analyze_account` | $0.001 USDC | `address: string` | Stellar `/accounts` (Horizon) |
| `get_weather` | $0.001 USDC | `city: string` | wttr.in (live) |
| `list_services` | FREE | — | Local service registry |

5 of 6 paid tools serve **Stellar-native data** directly from the network.

## 🔄 x402 Payment Flow

```
1. Agent calls API         →  GET /price
2. Server returns 402      ←  HTTP 402 + PAYMENT-REQUIRED header
                              {amount: "10000", asset: "USDC C...", payTo: "G..."}
3. Agent signs auth entry  →  ed25519 over SorobanAuthorizedInvocation
                              transfer(agent, server, 10000) on USDC SAC
4. Agent retries           →  GET /price + PAYMENT-SIGNATURE header
                              (base64 TransactionEnvelope with signed auth entry)
5. Server → Facilitator    →  POST /verify (OZ validates signature + balance)
6. Facilitator settles     →  Fee-bump tx submitted to Stellar testnet
                              USDC transferred: agent → server (on-chain)
7. Server returns data     ←  HTTP 200 + live response from Stellar DEX
```

## 💰 Revenue Model

**Per-call micropayments.** No subscriptions, no minimums, no API keys.

Any developer registers a service on-chain via the ServiceRegistry contract — sets a name, URL, and price in USDC base units. Agents worldwide discover it via `list_services` and pay per request. Settlement is instant on Stellar (~5 seconds), and fees are sub-cent.

This replaces the traditional `$500/month API subscription` model with `$0.001 per request` — making AI agent access economically viable at any scale.

## ⭐ Why Stellar

- **5-second finality** — payments settle before the HTTP timeout
- **Sub-cent fees** — micropayments are economically viable (try that on Ethereum)
- **USDC native** — Circle's USDC runs natively on Stellar via SAC
- **Soroban auth entries** — agents sign transfer authorizations without holding the transaction envelope
- **OZ Facilitator** — sponsors gas fees, so agents don't need XLM for fees

## 🎖️ What Makes This Different

1. **Real payments, not mocks** — Every API call settles real USDC on Stellar testnet. Verify on [Stellar Expert](https://stellar.expert/explorer/testnet/account/GCLLO5UEXQ5KS3WSSBFTZ4S3TJMJHKDCDIMU7WUIECSKDZJMEDX22AE7).
2. **Stellar-native data** — 5 of 6 endpoints serve live data from the Stellar DEX and Horizon. Not generic APIs.
3. **Full Soroban integration** — Two deployed contracts: ServiceRegistry for discovery, SmartAccount for spending policies.
4. **Session budgets** — Agent operates within a $0.01 spending cap. Budget tracked per-call with enforcement.
5. **MCP-native** — Plugs directly into Claude Desktop or Cursor. No custom client needed.
6. **Entirely Rust** — Contracts, server, MCP server, and demo all in Rust. One language, full stack.

## 📦 Quick Start

### 1. Generate and fund testnet accounts

```bash
stellar keys generate server --network testnet --fund
stellar keys generate agent  --network testnet --fund
```

### 2. Configure environment

```bash
cp .env.example .env
# Edit .env:
#   stellar keys show server    → SERVER_SECRET_KEY
#   stellar keys address server → SERVER_STELLAR_ADDRESS
#   stellar keys show agent     → AGENT_SECRET_KEY
#   stellar keys address agent  → AGENT_STELLAR_ADDRESS
#   OZ key from channels.openzeppelin.com/testnet/gen → FACILITATOR_API_KEY
```

### 3. Build and deploy contracts

```bash
make build
bash scripts/deploy-contracts.sh
```

### 4. Set up USDC trustlines and fund accounts

```bash
bash scripts/setup-agent.sh
# Visit https://faucet.circle.com → Stellar Testnet → fund AGENT and SERVER
```

### 5. Register services on-chain

```bash
bash scripts/register-services.sh
```

### 6. Run the demo

```bash
# Terminal 1
make dev-services

# Terminal 2
cargo run -p e2e-test
```

### 7. Connect to Claude (optional)

**Cursor:** `.cursor/mcp.json` is pre-configured — set `AGENT_SECRET_KEY` and `FACILITATOR_API_KEY` env vars.

**Claude Desktop:** See [docs/add-to-claude-desktop.md](docs/add-to-claude-desktop.md).

## 📋 Smart Contract Details

### ServiceRegistry ([`CAJZB47S...`](https://stellar.expert/explorer/testnet/contract/CAJZB47SSVIORFB4J7AHFVSA2HOSQU2YX4KJ5DORL4KK5EUX4AUSQJ4Y))

On-chain service catalog — 8 exported functions:

| Function | Description |
|----------|-------------|
| `register_service` | Register a new service (owner-signed) |
| `get_service` | Get service by ID |
| `get_all_services` | List all active services |
| `get_services_by_owner` | List services by owner address |
| `deregister_service` | Soft-delete (sets `active = false`) |
| `update_price` | Change price (owner-only) |
| `get_count` | Total registered services |

### SmartAccount ([`CCRXBYAV...`](https://stellar.expert/explorer/testnet/contract/CCRXBYAVZBCBYLYCZCJPLO22LYAIZGLR4QCJNFCHOZL2WAQJFWVGV7R3))

OZ `stellar-accounts` v0.6.0 smart wallet — 15 exported functions:

- Full `SmartAccount` trait: context rules, signers, policies
- `grant_agent_usdc_access(agent, usdc_contract, valid_until_ledger)` — scoped USDC access
- `__check_auth` → `do_check_auth` — Soroban custom account interface
- `Signer::Delegated(Address)` for G-address authorization
- `ContextRuleType::CallContract(usdc)` scopes auth to USDC only

## ⚠️ Known Limitations

- **Testnet only** — USDC has no real monetary value. Production requires pubnet passphrase and real USDC.
- **Local x402 server** — The payment-gated API runs locally. The Vercel dashboard shows live on-chain data but cannot trigger payments.
- **OZ Facilitator dependency** — Occasional 503s from the facilitator cause payment retries to fail. This is upstream, not our code.

## 🔗 Links

- [Stellar x402 docs](https://developers.stellar.org/docs/build/agentic-payments/x402)
- [x402 protocol spec](https://www.x402.org/)
- [OpenZeppelin Smart Accounts](https://docs.openzeppelin.com/stellar-contracts/accounts/smart-account)
- [OZ x402 Facilitator](https://github.com/OpenZeppelin/relayer-plugin-x402-facilitator)
- [rmcp — Rust MCP SDK](https://github.com/modelcontextprotocol/rust-sdk)
- [Stellar Hacks: Agents Hackathon](https://dorahacks.io/hackathon/stellar-agents-x402-stripe-mpp/detail)

## Prerequisites

- **Rust 1.80+** with `wasm32v1-none` target (`rustup target add wasm32v1-none`)
- **Node.js 22+**
- **Stellar CLI 25.x** (`stellar --version`)
- **OZ Facilitator API key** — [channels.openzeppelin.com/testnet/gen](https://channels.openzeppelin.com/testnet/gen)
- **Test USDC** — [faucet.circle.com](https://faucet.circle.com) (Stellar Testnet)
