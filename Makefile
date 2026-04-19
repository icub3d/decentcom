.PHONY: help clean setup dev servers client clean-identity

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

clean: ## Remove test databases, media dirs, and OS keychain test entries
	cargo run -q --bin test-setup -- clean

setup: clean ## Clean + bootstrap test DBs with users/roles/channels + store keys in keychain
	cd client && pnpm install
	cargo run -q --bin test-setup

clean-identity: ## Remove all decentcom keys from the OS keychain (useful during testing)
	@while secret-tool search application rust-keyring service decentcom 2>&1 | grep -q "attribute.username"; do \
		username=$$(secret-tool search application rust-keyring service decentcom 2>&1 | grep "attribute.username" | head -1 | awk '{print $$3}'); \
		echo "Deleting keychain entry: $$username"; \
		secret-tool clear application rust-keyring service decentcom target default username "$$username"; \
	done
	@echo "Keychain cleared."

dev: ## Start all 3 servers + client via overmind (requires overmind)
	overmind start -f Procfile

servers: ## Start only the 3 test servers (no client)
	overmind start -f Procfile -l open,private,strict

client: ## Start only the Tauri client dev server
	cd client && pnpm tauri dev

build: ## Build all workspace crates
	cargo build

test: ## Run all tests (server + shared + client)
	cargo test
	cd client && pnpm test

lint: ## Run clippy + eslint
	cargo clippy -- -D warnings
	cd client && pnpm lint
