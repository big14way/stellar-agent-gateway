"use client";
import { useState, useEffect, useCallback } from "react";
import {
    checkBackendHealth, explorerUrl, getBalances,
    getTransactions, truncate,
    type Balances, type TxRecord,
} from "@/lib/stellar";

const SERVICES = [
    {
        name: "Weather API",
        endpoint: "/weather",
        description: "Real-time weather data for any city",
        price: "$0.001",
        tags: ["weather", "data"],
    },
    {
        name: "XLM Price Feed",
        endpoint: "/price",
        description: "XLM/USDC exchange rate and 24h market data",
        price: "$0.001",
        tags: ["price", "defi"],
    },
    {
        name: "Account Analyzer",
        endpoint: "/account",
        description: "Stellar account balance and transaction analysis",
        price: "$0.002",
        tags: ["analytics", "stellar"],
    },
];

export default function Home() {
    const [walletAddress, setWalletAddress] = useState<string | null>(null);
    const [balances, setBalances]           = useState<Balances>({ xlm: "0", usdc: "0" });
    const [transactions, setTransactions]   = useState<TxRecord[]>([]);
    const [backendOk, setBackendOk]         = useState<boolean | null>(null);
    const [loading, setLoading]             = useState(false);
    const [walletError, setWalletError]     = useState<string | null>(null);

    useEffect(() => {
        checkBackendHealth().then(setBackendOk);
    }, []);

    const connectWallet = async () => {
        setWalletError(null);
        try {
            const { setAllowed } = await import("@stellar/freighter-api");
            const { isAllowed } = await setAllowed();
            if (!isAllowed) { setWalletError("Please allow access in Freighter."); return; }
            const { getAddress } = await import("@stellar/freighter-api");
            const { address } = await getAddress();
            setWalletAddress(address);
        } catch {
            setWalletError("Freighter not found. Install the browser extension.");
        }
    };

    const refreshData = useCallback(async () => {
        if (!walletAddress) return;
        setLoading(true);
        const [bal, txs] = await Promise.all([
            getBalances(walletAddress),
            getTransactions(walletAddress),
        ]);
        setBalances(bal);
        setTransactions(txs);
        setLoading(false);
    }, [walletAddress]);

    useEffect(() => { refreshData(); }, [refreshData]);
    useEffect(() => {
        if (!walletAddress) return;
        const id = setInterval(refreshData, 10000);
        return () => clearInterval(id);
    }, [walletAddress, refreshData]);

    return (
        <main className="max-w-7xl mx-auto px-4 py-8 space-y-8">

            {/* Header */}
            <header className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold">Stellar Agent Gateway</h1>
                    <p className="text-gray-400 mt-1 text-sm">
                        x402 AI Agent Marketplace · Stellar Testnet
                    </p>
                </div>
                <div className="flex items-center gap-3">
                    <span
                        className={`w-2.5 h-2.5 rounded-full ${
                            backendOk === true
                                ? "bg-green-400"
                                : backendOk === false
                                ? "bg-red-500"
                                : "bg-yellow-400 animate-pulse"
                        }`}
                    />
                    <span className="text-sm text-gray-400">
                        {backendOk === true
                            ? "Services online"
                            : backendOk === false
                            ? "Services offline — run: make dev-services"
                            : "Checking..."}
                    </span>
                    {!walletAddress ? (
                        <button
                            onClick={connectWallet}
                            className="bg-blue-600 hover:bg-blue-500 px-4 py-2 rounded-lg text-sm font-medium transition-colors"
                        >
                            Connect Freighter
                        </button>
                    ) : (
                        <a
                            href={explorerUrl("account", walletAddress)}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="bg-gray-800 px-4 py-2 rounded-lg text-sm font-mono text-green-400 hover:text-green-300"
                        >
                            {truncate(walletAddress)}
                        </a>
                    )}
                </div>
            </header>

            {walletError && (
                <div className="bg-red-900/30 border border-red-700 text-red-300 text-sm px-4 py-3 rounded-lg">
                    {walletError}
                </div>
            )}

            {/* Main grid */}
            <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">

                {/* Service Registry */}
                <section className="lg:col-span-2 space-y-3">
                    <h2 className="text-lg font-semibold text-gray-100">
                        Registered Services
                    </h2>
                    {SERVICES.map((svc) => (
                        <div
                            key={svc.name}
                            className="bg-gray-900 border border-gray-800 rounded-xl p-4"
                        >
                            <div className="flex items-start justify-between gap-4">
                                <div className="flex-1 min-w-0">
                                    <div className="font-semibold text-white">{svc.name}</div>
                                    <div className="text-gray-400 text-sm mt-0.5">
                                        {svc.description}
                                    </div>
                                    <div className="flex flex-wrap gap-1.5 mt-2">
                                        {svc.tags.map((t) => (
                                            <span
                                                key={t}
                                                className="text-xs bg-gray-800 text-gray-400 px-2 py-0.5 rounded"
                                            >
                                                {t}
                                            </span>
                                        ))}
                                        <span className="text-xs bg-blue-900/50 text-blue-300 px-2 py-0.5 rounded">
                                            x402
                                        </span>
                                        <span className="text-xs bg-green-900/50 text-green-400 px-2 py-0.5 rounded">
                                            Stellar testnet
                                        </span>
                                    </div>
                                </div>
                                <div className="text-yellow-400 font-mono font-bold text-sm whitespace-nowrap">
                                    {svc.price}/call
                                </div>
                            </div>
                        </div>
                    ))}
                    <p className="text-xs text-gray-600">
                        Services are registered on-chain via the ServiceRegistry Soroban contract.
                    </p>
                </section>

                {/* Agent Wallet */}
                <section className="space-y-3">
                    <h2 className="text-lg font-semibold text-gray-100">Agent Wallet</h2>
                    <div className="bg-gray-900 border border-gray-800 rounded-xl p-5 space-y-4">
                        {walletAddress ? (
                            <>
                                <div>
                                    <p className="text-xs text-gray-500 uppercase tracking-wider mb-1">
                                        Connected Address
                                    </p>
                                    <a
                                        href={explorerUrl("account", walletAddress)}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        className="text-blue-400 font-mono text-sm hover:text-blue-300 break-all"
                                    >
                                        {truncate(walletAddress, 10)}
                                    </a>
                                </div>
                                <div className="grid grid-cols-2 gap-3">
                                    <div className="bg-gray-800 rounded-lg p-3">
                                        <p className="text-xs text-gray-500">XLM</p>
                                        <p className="text-white font-semibold">
                                            {loading ? "..." : parseFloat(balances.xlm).toFixed(2)}
                                        </p>
                                    </div>
                                    <div className="bg-gray-800 rounded-lg p-3">
                                        <p className="text-xs text-gray-500">USDC</p>
                                        <p className="text-green-400 font-semibold">
                                            {loading ? "..." : parseFloat(balances.usdc || "0").toFixed(4)}
                                        </p>
                                    </div>
                                </div>
                                <button
                                    onClick={refreshData}
                                    disabled={loading}
                                    className="w-full bg-gray-800 hover:bg-gray-700 text-gray-300 text-sm py-2 rounded-lg transition-colors disabled:opacity-50"
                                >
                                    {loading ? "Refreshing..." : "Refresh Balances"}
                                </button>
                                <p className="text-xs text-gray-600">
                                    x402 payments debit USDC from this address.
                                    Budget limits enforced by Soroban SmartAccount.
                                </p>
                            </>
                        ) : (
                            <div className="text-center text-gray-500 py-6 space-y-2">
                                <p>Connect Freighter wallet</p>
                                <p className="text-xs">to view balances and transactions</p>
                            </div>
                        )}
                    </div>
                </section>
            </div>

            {/* Transaction Feed */}
            <section className="space-y-3">
                <div className="flex items-center justify-between">
                    <h2 className="text-lg font-semibold text-gray-100">
                        Transaction Feed
                    </h2>
                    {walletAddress && (
                        <span className="text-xs text-gray-500">auto-refresh every 10s</span>
                    )}
                </div>
                {!walletAddress ? (
                    <div className="bg-gray-900 border border-gray-800 rounded-xl p-10 text-center text-gray-500">
                        Connect wallet to see x402 payment transactions
                    </div>
                ) : transactions.length === 0 ? (
                    <div className="bg-gray-900 border border-gray-800 rounded-xl p-10 text-center text-gray-500">
                        {loading
                            ? "Loading transactions..."
                            : "No transactions yet. Call a tool in Claude to trigger a payment."}
                    </div>
                ) : (
                    <div className="bg-gray-900 border border-gray-800 rounded-xl overflow-hidden">
                        <table className="w-full text-sm">
                            <thead>
                                <tr className="border-b border-gray-800">
                                    <th className="text-left px-4 py-3 text-gray-500 font-medium">
                                        Hash
                                    </th>
                                    <th className="text-left px-4 py-3 text-gray-500 font-medium">
                                        Time
                                    </th>
                                    <th className="text-left px-4 py-3 text-gray-500 font-medium">
                                        Ops
                                    </th>
                                </tr>
                            </thead>
                            <tbody>
                                {transactions.map((tx) => (
                                    <tr
                                        key={tx.id}
                                        className="border-b border-gray-800/40 hover:bg-gray-800/30"
                                    >
                                        <td className="px-4 py-3">
                                            <a
                                                href={explorerUrl("tx", tx.hash)}
                                                target="_blank"
                                                rel="noopener noreferrer"
                                                className="text-blue-400 hover:text-blue-300 font-mono"
                                            >
                                                {truncate(tx.hash, 10)}
                                            </a>
                                        </td>
                                        <td className="px-4 py-3 text-gray-400">
                                            {new Date(tx.created_at).toLocaleString()}
                                        </td>
                                        <td className="px-4 py-3 text-gray-400">
                                            {tx.operation_count}
                                        </td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                )}
            </section>

            <footer className="text-center text-xs text-gray-700 pt-4">
                Stellar Hacks: Agents Hackathon · x402 V2 · Soroban smart contracts ·{" "}
                <a
                    href="https://dorahacks.io/hackathon/stellar-agents-x402-stripe-mpp/detail"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="hover:text-gray-500"
                >
                    DoraHacks
                </a>
            </footer>
        </main>
    );
}
