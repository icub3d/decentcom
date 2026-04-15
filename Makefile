.PHONY: help clean setup dev servers client

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

clean: ## Remove test databases, media dirs, and OS keychain test entries
	python3 scripts/test-setup.py --clean

setup: clean ## Clean + bootstrap test DBs with users/roles/channels + store keys in keychain
	python3 scripts/test-setup.py

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
