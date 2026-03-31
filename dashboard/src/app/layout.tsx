import type { Metadata } from "next";
import "./globals.css";
export const metadata: Metadata = {
  title: "Stellar Agent Gateway",
  description: "x402 AI Agent Marketplace on Stellar",
};
export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body className="bg-gray-950 text-white min-h-screen">{children}</body>
    </html>
  );
}
