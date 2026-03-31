export const HORIZON_URL   = "https://horizon-testnet.stellar.org";
export const TESTNET_RPC   = "https://soroban-testnet.stellar.org";
export const USDC_CONTRACT = "CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA";
export const SERVICES_URL  = "http://localhost:3001";

export function explorerUrl(type: "tx" | "account" | "contract", id: string): string {
    return `https://stellar.expert/explorer/testnet/${type}/${id}`;
}

export function truncate(s: string, chars = 8): string {
    if (!s || s.length <= chars * 2 + 3) return s;
    return `${s.slice(0, chars)}...${s.slice(-chars)}`;
}

export interface Balances { xlm: string; usdc: string; }
export interface TxRecord { id: string; hash: string; created_at: string; operation_count: number; }

export async function getBalances(address: string): Promise<Balances> {
    try {
        const res = await fetch(`${HORIZON_URL}/accounts/${address}`);
        if (!res.ok) return { xlm: "0", usdc: "0" };
        const data = await res.json();
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const xlm  = data.balances?.find((b: any) => b.asset_type === "native")?.balance ?? "0";
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const usdc = data.balances?.find((b: any) => b.asset_code === "USDC")?.balance ?? "0";
        return { xlm, usdc };
    } catch { return { xlm: "0", usdc: "0" }; }
}

export async function getTransactions(address: string, limit = 10): Promise<TxRecord[]> {
    try {
        const res = await fetch(
            `${HORIZON_URL}/accounts/${address}/transactions?limit=${limit}&order=desc`
        );
        if (!res.ok) return [];
        const data = await res.json();
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        return (data._embedded?.records ?? []).map((r: any) => ({
            id: r.id, hash: r.hash, created_at: r.created_at,
            operation_count: r.operation_count,
        }));
    } catch { return []; }
}

export async function checkBackendHealth(): Promise<boolean> {
    try {
        const res = await fetch(`${SERVICES_URL}/health`, {
            signal: AbortSignal.timeout(3000),
        });
        return res.ok;
    } catch { return false; }
}
