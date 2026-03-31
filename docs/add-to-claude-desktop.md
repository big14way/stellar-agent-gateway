# Add Stellar Agent Gateway to Claude Desktop

## Build the binary

```bash
cargo build --release -p stellar-mcp-server
# Binary location: ./target/release/stellar-mcp-server
```

## Edit Claude Desktop config

**macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows:** `%APPDATA%\Claude\claude_desktop_config.json`

Add this (replace `/FULL/PATH/TO/` with your actual project path):

```json
{
  "mcpServers": {
    "stellar-agent-gateway": {
      "command": "/FULL/PATH/TO/stellar-agent-gateway/target/release/stellar-mcp-server",
      "args": [],
      "env": {
        "RUST_LOG": "warn",
        "AGENT_SECRET_KEY": "S...",
        "STELLAR_RPC_URL": "https://soroban-testnet.stellar.org",
        "FACILITATOR_URL": "https://channels.openzeppelin.com/x402/testnet",
        "FACILITATOR_API_KEY": "your_api_key_here"
      }
    }
  }
}
```

## Restart Claude Desktop

Quit completely (Cmd+Q on Mac, not just close window), then relaunch.

## Verify

Type: "list gateway services"

Claude should call `list_services` and return the 4 tools with descriptions and prices.
