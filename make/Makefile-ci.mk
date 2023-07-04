.PHONY: fmt
fmt: ## Run format check
	$(Q) echo "$(M) Run formatter"
	$(Q) cargo fmt -- --check
	$(Q) echo "$(M) Formatter done!"

.PHONY: lint
lint: ${DEV_DB_FILE} ## Run lint check
	$(Q) echo "$(M) Run linter"
	$(Q) cargo clippy -- -D warnings
	$(Q) echo "$(M) Run Linter done!"

.PHONY: test
test: ${DEV_DB_FILE} ## Run tests with coverage report
	$(Q) echo "$(M) Run tests"
	$(Q) cargo tarpaulin --ignore-tests --workspace --timeout 120 --skip-clean --out Xml
	$(Q) echo "$(M) Tests passed!"

.PHONY: audit
audit: ## Run audit on dependencies
	$(Q) echo "$(M) Run audit"
	$(Q) cargo audit
	$(Q) echo "$(M) Audit passed!"

.PHONY: ci
ci: fmt lint audit test ## Run all CI steps
	$(Q) echo "$(M) CI done!"
