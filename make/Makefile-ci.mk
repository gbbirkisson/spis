.PHONY: fmt
fmt: ## Run format check
	$(info $(M) Run formatter)
	$(Q) cargo fmt -- --check
	$(info $(M) Formatter done!)

.PHONY: lint
lint: ${DEV_DB_FILE} ## Run lint check
	$(info $(M) Run linter)
	$(Q) cargo clippy -- -D warnings
	$(info $(M) Linter done!)

.PHONY: test
test: ${DEV_DB_FILE} ## Run tests with coverage report
	$(info $(M) Run tests)
	$(Q) cargo tarpaulin --version > /dev/null || cargo install cargo-tarpaulin
	$(Q) cargo tarpaulin --ignore-tests --workspace --timeout 120 --skip-clean --out Xml
	$(info $(M) Tests passed!)

.PHONY: audit
audit: ## Run audit on dependencies
	$(info $(M) Run audit)
	$(Q) cargo audit --version > /dev/null || cargo install cargo-audit
	$(Q) cargo audit
	$(info $(M) Audit passed!)

.PHONY: ci
ci: fmt lint audit test ## Run all CI steps
	$(info $(M) CI done!)
