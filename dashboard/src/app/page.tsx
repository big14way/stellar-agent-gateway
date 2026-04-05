"use client";
import { useState, useEffect, useCallback, useRef } from "react";
import {
    AGENT_ADDRESS, SERVER_ADDRESS, SERVICE_REGISTRY, SMART_ACCOUNT,
    checkBackendHealth, explorerUrl, getBalances,
    getTransactions, truncate,
    type Balances, type TxRecord,
} from "@/lib/stellar";

const SERVICES = [
    {
        name: "XLM Price Feed",
        endpoint: "/price",
        description: "Live XLM/USDC rate from Stellar DEX orderbook",
        price: "$0.001",
        tags: ["DEX", "price", "defi"],
    },
    {
        name: "DEX Trade History",
        endpoint: "/dex-trades",
        description: "Recent XLM/USDC trades with prices and amounts",
        price: "$0.001",
        tags: ["DEX", "trades", "stellar"],
    },
    {
        name: "Liquidity Pools",
        endpoint: "/liquidity-pools",
        description: "AMM pool reserves, fees, and share stats",
        price: "$0.001",
        tags: ["AMM", "liquidity", "defi"],
    },
    {
        name: "Network Stats",
        endpoint: "/network-stats",
        description: "Latest ledger, base fee, protocol version",
        price: "$0.001",
        tags: ["network", "ledger", "fees"],
    },
    {
        name: "Account Analyzer",
        endpoint: "/account",
        description: "Live account balances and trustlines from Horizon",
        price: "$0.001",
        tags: ["analytics", "stellar"],
    },
    {
        name: "Weather API",
        endpoint: "/weather?city=Lagos",
        description: "Live weather data from wttr.in",
        price: "$0.001",
        tags: ["weather", "live data"],
    },
];

function BalanceCard({ label, xlm, usdc, prev, loading }: {
    label: string; xlm: string; usdc: string; prev: Balances | null; loading: boolean;
}) {
    const usdcNow  = parseFloat(usdc || "0");
    const usdcPrev = prev ? parseFloat(prev.usdc || "0") : usdcNow;
    const diff = usdcNow - usdcPrev;
    const changed = Math.abs(diff) > 0.0000001 && prev !== null;

    return (
        <div className={`bg-gray-800 rounded-lg p-3 transition-all duration-500 ${changed ? "ring-2 ring-green-400/50" : ""}`}>
            <p className="text-xs text-gray-500 mb-2">{label}</p>
            <div className="flex items-baseline gap-2">
                <span className="text-green-400 font-bold text-lg">
                    {loading ? "..." : usdcNow.toFixed(7)}
                </span>
                <span className="text-xs text-gray-500">USDC</span>
            </div>
            {changed && (
                <span className={`text-xs font-mono ${diff > 0 ? "text-green-400" : "text-red-400"}`}>
                    {diff > 0 ? "+" : ""}{diff.toFixed(7)}
                </span>
            )}
            <div className="flex items-baseline gap-2 mt-1">
                <span className="text-white font-semibold text-sm">
                    {loading ? "..." : parseFloat(xlm).toFixed(2)}
                </span>
                <span className="text-xs text-gray-500">XLM</span>
            </div>
        </div>
    );
}

export default function Home() {
    const [agentBal, setAgentBal]       = useState<Balances>({ xlm: "0", usdc: "0" });
    const [serverBal, setServerBal]     = useState<Balances>({ xlm: "0", usdc: "0" });
    const [prevAgentBal, setPrevAgent]  = useState<Balances | null>(null);
    const [prevServerBal, setPrevServer] = useState<Balances | null>(null);
    const [transactions, setTransactions] = useState<TxRecord[]>([]);
    const [backendOk, setBackendOk]     = useState<boolean | null>(null);
    const [loading, setLoading]         = useState(true);
    const [txCount, setTxCount]         = useState(0);
    const prevTxIds = useRef<Set<string>>(new Set());

    const refreshData = useCallback(async () => {
        const [aBal, sBal, txs, health] = await Promise.all([
            getBalances(AGENT_ADDRESS),
            getBalances(SERVER_ADDRESS),
            getTransactions(AGENT_ADDRESS),
            checkBackendHealth(),
        ]);

        setPrevAgent(prev => prev === null ? aBal : agentBal);
        setPrevServer(prev => prev === null ? sBal : serverBal);
        setAgentBal(aBal);
        setServerBal(sBal);
        setBackendOk(health);

        // Mark new transactions
        const marked = txs.map(tx => ({
            ...tx,
            is_new: !prevTxIds.current.has(tx.id) && prevTxIds.current.size > 0,
        }));
        setTransactions(marked);
        if (txs.length !== txCount) setTxCount(txs.length);
        prevTxIds.current = new Set(txs.map(t => t.id));

        setLoading(false);
    // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [txCount]);

    useEffect(() => { refreshData(); }, [refreshData]);
    useEffect(() => {
        const id = setInterval(refreshData, 5000);
        return () => clearInterval(id);
    }, [refreshData]);

    return (
        <main className="max-w-7xl mx-auto px-4 py-8 space-y-6">

            {/* Header */}
            <header className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold">Stellar Agent Gateway</h1>
                    <p className="text-gray-400 mt-1 text-sm">
                        x402 Micropayment Marketplace · Stellar Testnet · Live
                    </p>
                </div>
                <div className="flex items-center gap-3">
                    <span className={`w-2.5 h-2.5 rounded-full ${
                        backendOk === true ? "bg-green-400"
                            : backendOk === false ? "bg-red-500"
                            : "bg-yellow-400 animate-pulse"
                    }`} />
                    <span className="text-sm text-gray-400">
                        {backendOk === true ? "x402 services online"
                            : backendOk === false ? "Services offline"
                            : "Checking..."}
                    </span>
                    <span className="text-xs text-gray-600 font-mono">5s refresh</span>
                </div>
            </header>

            {/* Accounts + Services Grid */}
            <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">

                {/* Agent & Server Wallets */}
                <section className="space-y-3">
                    <h2 className="text-lg font-semibold text-gray-100">Live Accounts</h2>

                    {/* Agent */}
                    <div className="bg-gray-900 border border-gray-800 rounded-xl p-4 space-y-3">
                        <div className="flex items-center justify-between">
                            <span className="text-xs text-blue-400 uppercase tracking-wider font-semibold">Agent (payer)</span>
                            <a href={explorerUrl("account", AGENT_ADDRESS)} target="_blank" rel="noopener noreferrer"
                                className="text-xs text-gray-500 hover:text-blue-400 font-mono">
                                {truncate(AGENT_ADDRESS, 6)}
                            </a>
                        </div>
                        <BalanceCard label="Agent Balance" xlm={agentBal.xlm} usdc={agentBal.usdc} prev={prevAgentBal} loading={loading} />
                        <p className="text-xs text-gray-600">USDC decreases when agent pays for API calls</p>
                    </div>

                    {/* Server */}
                    <div className="bg-gray-900 border border-gray-800 rounded-xl p-4 space-y-3">
                        <div className="flex items-center justify-between">
                            <span className="text-xs text-yellow-400 uppercase tracking-wider font-semibold">Server (receiver)</span>
                            <a href={explorerUrl("account", SERVER_ADDRESS)} target="_blank" rel="noopener noreferrer"
                                className="text-xs text-gray-500 hover:text-blue-400 font-mono">
                                {truncate(SERVER_ADDRESS, 6)}
                            </a>
                        </div>
                        <BalanceCard label="Server Balance" xlm={serverBal.xlm} usdc={serverBal.usdc} prev={prevServerBal} loading={loading} />
                        <p className="text-xs text-gray-600">USDC increases when payments are settled on-chain</p>
                    </div>

                    {/* Contracts */}
                    <div className="bg-gray-900 border border-gray-800 rounded-xl p-4 space-y-2">
                        <span className="text-xs text-gray-500 uppercase tracking-wider font-semibold">Soroban Contracts</span>
                        <div className="space-y-1.5">
                            <div className="flex items-center justify-between">
                                <span className="text-xs text-gray-400">ServiceRegistry</span>
                                <a href={explorerUrl("contract", SERVICE_REGISTRY)} target="_blank" rel="noopener noreferrer"
                                    className="text-xs text-blue-400 hover:text-blue-300 font-mono">
                                    {truncate(SERVICE_REGISTRY, 6)}
                                </a>
                            </div>
                            <div className="flex items-center justify-between">
                                <span className="text-xs text-gray-400">SmartAccount</span>
                                <a href={explorerUrl("contract", SMART_ACCOUNT)} target="_blank" rel="noopener noreferrer"
                                    className="text-xs text-blue-400 hover:text-blue-300 font-mono">
                                    {truncate(SMART_ACCOUNT, 6)}
                                </a>
                            </div>
                        </div>
                    </div>
                </section>

                {/* Service Registry + Transaction Feed */}
                <section className="lg:col-span-2 space-y-6">

                    {/* Services */}
                    <div className="space-y-3">
                        <h2 className="text-lg font-semibold text-gray-100">
                            x402 Payment-Gated Services
                        </h2>
                        {SERVICES.map((svc) => (
                            <div key={svc.name} className="bg-gray-900 border border-gray-800 rounded-xl p-4">
                                <div className="flex items-start justify-between gap-4">
                                    <div className="flex-1 min-w-0">
                                        <div className="font-semibold text-white">{svc.name}</div>
                                        <div className="text-gray-400 text-sm mt-0.5">{svc.description}</div>
                                        <div className="flex flex-wrap gap-1.5 mt-2">
                                            {svc.tags.map((t) => (
                                                <span key={t} className="text-xs bg-gray-800 text-gray-400 px-2 py-0.5 rounded">{t}</span>
                                            ))}
                                            <span className="text-xs bg-blue-900/50 text-blue-300 px-2 py-0.5 rounded">x402</span>
                                            <span className="text-xs bg-purple-900/50 text-purple-300 px-2 py-0.5 rounded">Soroban auth</span>
                                        </div>
                                    </div>
                                    <div className="text-right">
                                        <div className="text-yellow-400 font-mono font-bold text-sm">{svc.price}</div>
                                        <div className="text-xs text-gray-600">per call</div>
                                    </div>
                                </div>
                            </div>
                        ))}
                    </div>

                    {/* Transaction Feed */}
                    <div className="space-y-3">
                        <div className="flex items-center justify-between">
                            <h2 className="text-lg font-semibold text-gray-100">
                                Agent Transaction Feed
                            </h2>
                            <span className="text-xs text-gray-500">
                                {transactions.length} transactions · live
                            </span>
                        </div>
                        {transactions.length === 0 ? (
                            <div className="bg-gray-900 border border-gray-800 rounded-xl p-10 text-center text-gray-500">
                                {loading
                                    ? "Loading transactions..."
                                    : "No transactions yet. Run the demo: cargo run -p e2e-test"}
                            </div>
                        ) : (
                            <div className="bg-gray-900 border border-gray-800 rounded-xl overflow-hidden">
                                <table className="w-full text-sm">
                                    <thead>
                                        <tr className="border-b border-gray-800">
                                            <th className="text-left px-4 py-3 text-gray-500 font-medium">Tx Hash</th>
                                            <th className="text-left px-4 py-3 text-gray-500 font-medium">Time</th>
                                            <th className="text-left px-4 py-3 text-gray-500 font-medium">Ops</th>
                                            <th className="text-left px-4 py-3 text-gray-500 font-medium">Fee</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {transactions.map((tx) => (
                                            <tr key={tx.id}
                                                className={`border-b border-gray-800/40 transition-all duration-1000 ${
                                                    tx.is_new ? "bg-green-900/20" : "hover:bg-gray-800/30"
                                                }`}>
                                                <td className="px-4 py-3">
                                                    <a href={explorerUrl("tx", tx.hash)} target="_blank" rel="noopener noreferrer"
                                                        className="text-blue-400 hover:text-blue-300 font-mono">
                                                        {truncate(tx.hash, 8)}
                                                    </a>
                                                </td>
                                                <td className="px-4 py-3 text-gray-400">
                                                    {new Date(tx.created_at).toLocaleTimeString()}
                                                </td>
                                                <td className="px-4 py-3 text-gray-400">{tx.operation_count}</td>
                                                <td className="px-4 py-3 text-gray-500 font-mono text-xs">
                                                    {(parseInt(tx.fee_charged) / 10_000_000).toFixed(4)} XLM
                                                </td>
                                            </tr>
                                        ))}
                                    </tbody>
                                </table>
                            </div>
                        )}
                    </div>
                </section>
            </div>

            {/* How it works */}
            <section className="bg-gray-900/50 border border-gray-800 rounded-xl p-6">
                <h2 className="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-3">How x402 Payment Flow Works</h2>
                <div className="grid grid-cols-1 md:grid-cols-5 gap-3 text-center text-xs">
                    {[
                        { step: "1", label: "Agent calls API", detail: "GET /weather?city=Lagos" },
                        { step: "2", label: "Server returns 402", detail: "Payment Required + USDC amount" },
                        { step: "3", label: "Agent signs auth", detail: "Soroban ed25519 auth entry" },
                        { step: "4", label: "OZ Facilitator settles", detail: "Fee-bump tx → Stellar testnet" },
                        { step: "5", label: "Data returned", detail: "HTTP 200 + live response" },
                    ].map((s) => (
                        <div key={s.step} className="bg-gray-800/50 rounded-lg p-3">
                            <div className="w-6 h-6 rounded-full bg-blue-600 text-white font-bold text-xs flex items-center justify-center mx-auto mb-2">{s.step}</div>
                            <div className="text-gray-200 font-medium">{s.label}</div>
                            <div className="text-gray-500 mt-1">{s.detail}</div>
                        </div>
                    ))}
                </div>
            </section>

            <footer className="text-center text-xs text-gray-700 pt-2">
                Stellar Hacks: Agents Hackathon · x402 V2 · Soroban Smart Contracts · OZ Facilitator ·{" "}
                <a href="https://dorahacks.io/hackathon/stellar-agents-x402-stripe-mpp/detail"
                    target="_blank" rel="noopener noreferrer" className="hover:text-gray-500">
                    DoraHacks
                </a>
            </footer>
        </main>
    );
}
