.PHONY: build deploy setup-agent services dev-services dev-mcp dev-dashboard test

build:
	stellar contract build
	cargo build --release -p x402-services
	cargo build --release -p stellar-mcp-server

deploy:
	bash scripts/deploy-contracts.sh

setup-agent:
	bash scripts/setup-agent.sh

services:
	bash scripts/register-services.sh

dev-services:
	RUST_LOG=info cargo run -p x402-services

dev-mcp:
	RUST_LOG=stellar_mcp_server=info cargo run -p stellar-mcp-server

dev-dashboard:
	cd dashboard && npm run dev

test:
	cargo test --workspace
	bash scripts/test-integration.sh
