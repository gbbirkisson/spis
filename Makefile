.DEFAULT_GOAL:=help

NGINX_CONF:=dev/nginx.conf
NGINX_CONF_TMP:=/tmp/nginx.conf

dev/api/state/thumbnails:
	mkdir -p dev/api/state/thumbnails

.PHONY: _dev-run-nginx-nowatch
_dev-run-nginx-nowatch:
	cat $(NGINX_CONF) | envsubst > $(NGINX_CONF_TMP)
	nginx -c $(NGINX_CONF_TMP)

.PHONY: fmt
fmt: ## Run format check
	cargo fmt -- --check

.PHONY: lint
lint: ## Run lint check
	cargo clippy -- -D warnings

.PHONY: test-cargo
test-cargo: ## Run cargo tests
	cargo test

.PHONY: test-coverage
test-coverage: ## Run tests with coverage report
	cargo install cargo-tarpaulin
	cargo tarpaulin --ignore-tests --verbose --all-features --workspace --timeout 120 --out Xml

.PHONY: test
test: test-cargo test-coverage ## Run tests

audit: ## Run audit on dependencies
	cargo install cargo-audit
	cargo audit

.PHONY: ci
ci: fmt lint test audit ## Run CI steps

.PHONY: dev
dev: ## Run all dev processes
	x-terminal-emulator -t nginx -e make dev-nginx &
	x-terminal-emulator -t server -e make dev-server &
	x-terminal-emulator -t gui -e make dev-gui &
	xdg-open http://localhost:9000

.PHONY: dev-nginx
dev-nginx: ## Run nginx
	watchexec -r -w dev/nginx.conf -- make _dev-run-nginx-nowatch

.PHONY: dev-server
dev-server: dev/api/state/thumbnails ## Run the server
	watchexec -r -e rs,toml -w spis-model -w spis-server -- cargo run -p spis-server

.PHONY: dev-gui
dev-gui: ## Run the gui
	cd spis-gui && trunk serve --port 9000 --proxy-backend http://localhost:7000/api/

.PHONY: dl-img
dl-img: ## Download random images
	./dev/images.sh 100 dev/api/images

.PHONY: setup
setup: ## Setup project dependencies
	# Install cargo binaries
	cargo install watchexec-cli
	cargo install trunk
	cargo install cargo-watch

	# Add rust components/targets
	rustup component add rustfmt
	rustup component add clippy
	rustup target add wasm32-unknown-unknown

	# Install apt packages
	sudo apt install -y nginx

.PHONY: help
help: ## Show this help
	$(eval HELP_COL_WIDTH:=15)
	@echo "Makefile targets:"
	@grep -E '[^\s]+:.*?## .*$$' ${MAKEFILE_LIST} | grep -v grep | envsubst | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-${HELP_COL_WIDTH}s\033[0m %s\n", $$1, $$2}'
