# Stellar Agent Gateway — 3-Minute Demo Video Pitch Script

## Pre-Recording Setup

**Terminal 1:** `make dev-services` (server running)
**Terminal 2:** Ready to run `cargo run -p e2e-test`
**Browser:** `http://localhost:3000` (dashboard open)
**Screen layout:** Split — terminal left, dashboard right
**Recording tool:** QuickTime (Cmd+Shift+5) or OBS

---

## The Script

### OPENING — Problem (0:00 - 0:40)

> **Screen: Show a blank terminal with just your cursor blinking**

*"Every AI agent today is locked behind API keys.*

*If you're a developer in Lagos building an AI assistant, and you want it to access weather data, market prices, and blockchain analytics — you need to sign up for three different services, get approved for API keys, set up billing with a credit card, and manage rate limits. Your agent can reason, plan, and act — but it can't pay.*

*This is the bottleneck holding back the agent economy. Not intelligence — access.*

*We built Stellar Agent Gateway to fix this."*

> **Screen: Type and hit enter on `cargo run -p e2e-test` — pause as the banner appears**

---

### THE SOLUTION — What We Built (0:40 - 1:10)

> **Screen: The demo banner and preflight checks are now visible. Point to the dashboard on the right showing both accounts and services.**

*"Stellar Agent Gateway is an x402-powered marketplace where AI agents autonomously pay for API services using real USDC micropayments on Stellar.*

*No API keys. No billing accounts. No human in the loop.*

*The agent discovers a service, gets an HTTP 402 — Payment Required — signs a Soroban authorization entry with its own key, and the OpenZeppelin Facilitator settles the payment on-chain. All in under 5 seconds."*

> **Screen: Point to the dashboard — show the agent balance (USDC), server balance, and the three service cards with prices.**

*"On the left — our agent's wallet. On the right — three live API services, each gated by x402. Let's watch the agent work."*

---

### LIVE DEMO — The Wow Moment (1:10 - 2:10)

> **Screen: The e2e test is now running Step 1 (XLM Price). Watch the terminal narrate each step.**

*"The agent is asked to analyze a Stellar portfolio. It autonomously decides it needs three pieces of data.*

*First — the live XLM price. It calls our price API, gets a 402, signs a Soroban auth entry — that's real ed25519 cryptography over the USDC transfer — retries with the payment, and the Facilitator settles it on Stellar testnet."*

> **Screen: Step 1 completes — show the live XLM price from the Stellar DEX orderbook appearing in the terminal. Glance at the dashboard — agent USDC has decreased.**

*"That price came live from the Stellar DEX orderbook. Not mocked. And the agent just paid one-tenth of a cent."*

> **Screen: Step 2 runs — account analysis. Step 3 runs — weather.**

*"Now it pulls the account's real balances from Horizon, then live weather data from Lagos. Three API calls, three micropayments, all settled on-chain."*

> **Screen: The portfolio report appears in the terminal. On the dashboard, both balances have updated and new transactions are highlighted green.**

*"The agent synthesizes everything into a portfolio report. Total cost — three-tenths of a cent. And look at the dashboard — you can see the USDC moving in real time. Agent balance went down, server balance went up. Every transaction is verifiable on Stellar Expert."*

> **Screen: Click one of the Stellar Expert links in the terminal output — show the real transactions on-chain in the browser.**

---

### ARCHITECTURE — Why Stellar (2:10 - 2:40)

> **Screen: Show the dashboard's "How x402 Payment Flow Works" diagram at the bottom.**

*"Under the hood — two Soroban smart contracts. A ServiceRegistry for on-chain service discovery, and an OpenZeppelin SmartAccount for programmable spending policies.*

*The MCP server speaks JSON-RPC over stdio — it plugs directly into Claude, Cursor, or any MCP-compatible AI. The x402 services server runs on Axum. Everything is written in Rust.*

*We chose Stellar because five-second finality and sub-cent fees make micropayments actually viable. You can't do one-tenth-of-a-cent payments on Ethereum — but on Stellar, it just works."*

---

### CLOSE — Vision (2:40 - 3:00)

> **Screen: Show the dashboard with updated balances — proof that real money moved.**

*"Today, we demonstrated three API calls. But the protocol scales to thousands — any developer can register a service on-chain, set a price, and agents worldwide can discover and pay for it instantly.*

*No gatekeepers. No API keys. No credit cards. Just software paying software, on Stellar.*

*This is Stellar Agent Gateway. Thank you."*

> **Screen: Hold on the dashboard for 3 seconds, then end recording.**

---

## Recording Checklist

| Order | What to show | When |
|-------|-------------|------|
| 1 | Blank terminal (cursor blinking) | Opening narration |
| 2 | Type `cargo run -p e2e-test` and hit enter | "We built Stellar Agent Gateway" |
| 3 | Dashboard (split-screen right side) | Show accounts + services |
| 4 | Terminal Step 1 completing — live price data | "Live from the Stellar DEX" |
| 5 | Dashboard balance changing (green highlight) | "USDC moving in real time" |
| 6 | Terminal portfolio report | "Three-tenths of a cent" |
| 7 | Click Stellar Expert link — show real on-chain tx | "Verifiable on Stellar Expert" |
| 8 | Dashboard "How it works" diagram | Architecture section |
| 9 | Hold on dashboard with final balances | Closing |

## Tips for Recording

- **Speak slowly and confidently** — judges watch at 1x speed
- **Pause 2 seconds** after each payment settles so the dashboard update is visible
- **Don't read the terminal output** — narrate what's happening conceptually
- **The Stellar Expert page is your proof** — show it for at least 5 seconds
- **End on the dashboard** — it's the most visual thing you have
