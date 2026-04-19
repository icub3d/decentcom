.PHONY: help clean setup dev servers client clean-identity seed build test lint

# `seed` is a one-shot process — let overmind treat its clean exit as success
# instead of tearing the whole session down.
OVERMIND_DEV_ENV := OVERMIND_CAN_DIE=seed

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

clean: ## Remove test databases, media dirs, WebView storage, and OS keychain test entries
	cargo run -q --bin test-setup -- clean

setup: ## Clean + install client deps + stage keychain entries and WebView localStorage
	cd client && pnpm install
	cargo run -q --bin test-setup -- prepare

clean-identity: ## Remove all decentcom keys from the OS keychain (useful during testing)
	@while secret-tool search application rust-keyring service decentcom 2>&1 | grep -q "attribute.username"; do \
		username=$$(secret-tool search application rust-keyring service decentcom 2>&1 | grep "attribute.username" | head -1 | awk '{print $$3}'); \
		echo "Deleting keychain entry: $$username"; \
		secret-tool clear application rust-keyring service decentcom target default username "$$username"; \
	done
	@echo "Keychain cleared."

dev: ## Start the 3 servers, run sdk-seed once they're up, then start the client
	$(OVERMIND_DEV_ENV) overmind start -f Procfile

servers: ## Start only the 3 test servers (no client, no seed)
	overmind start -f Procfile.servers

seed: ## Run the SDK-based seeder against already-running servers
	cargo run -q --bin sdk-seed

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
