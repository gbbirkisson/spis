.PHONY: fmt
fmt: ## Run format check
	cargo fmt -- --check

.PHONY: lint
lint: ${DEV_DB_FILE} ## Run lint check
	cargo clippy -- -D warnings

.PHONY: test
test: ${DEV_DB_FILE} ## Run tests with coverage report
	cargo tarpaulin --version > /dev/null || cargo install cargo-tarpaulin
	cargo tarpaulin --ignore-tests --workspace --timeout 120 --skip-clean --out Xml

audit: ## Run audit on dependencies
	cargo audit --version > /dev/null || cargo install cargo-audit
	cargo audit

.PHONY: ci
ci: fmt lint audit test ## Run CI steps