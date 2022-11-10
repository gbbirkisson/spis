.DEFAULT_GOAL:=help

NGINX_CONF:=dev/nginx.conf
NGINX_CONF_TMP:=/tmp/nginx.conf

BASE_DIR:=dev/api
IMAGE_DIR:=${BASE_DIR}/images
THUMBNAIL_DIR:=${BASE_DIR}/state/thumbnails
DB_FILE:=${BASE_DIR}/state/spis.db


${THUMBNAIL_DIR}:
	mkdir -p ${THUMBNAIL_DIR}

${IMAGE_DIR}:
	mkdir -p ${IMAGE_DIR}
	$(MAKE) dl-img

${DB_FILE}:
	sqlx database create
	sqlx migrate run --source spis-server/migrations

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

.PHONY: test-nocov
test-nocov: ## Run tests with no coverage report
	cargo test

.PHONY: test
test: ## Run tests with coverage report
	cargo tarpaulin --version > /dev/null || cargo install cargo-tarpaulin
	cargo tarpaulin --ignore-tests --all-features --workspace --timeout 120 --skip-clean --out Xml

audit: ## Run audit on dependencies
	cargo audit --version > /dev/null || cargo install cargo-audit
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
dev-server: ${IMAGE_DIR} ${THUMBNAIL_DIR} ${DB_FILE} ## Run the server
	watchexec -r -e rs,toml -w spis-model -w spis-server -- cargo run -p spis-server

.PHONY: dev-gui
dev-gui: ## Run the gui
	cd spis-gui && trunk serve --port 9000 --proxy-backend http://localhost:7000/api/

TEST_PROJ?=spis-server
TEST_NAME?=
.PHONY: dev-test
dev-test: ## Run specific test
	test ${TEST_NAME} || (echo "Set env var TEST_NAME to specify test name!"; exit 1)
	watchexec -r -e rs,toml -w ${TEST_PROJ} -- cargo test -q -p ${TEST_PROJ} ${TEST_NAME} -- --nocapture

.PHONY: dl-img
dl-img: ${IMAGE_DIR} ## Download 20 random images
	./dev/images.sh 20 ${IMAGE_DIR}

.PHONY: setup
setup: ${IMAGE_DIR} ${THUMBNAIL_DIR} ## Setup project dependencies and dirs
	# Install cargo binaries
	cargo install watchexec-cli
	cargo install trunk
	cargo install cargo-watch
	cargo install sqlx-cli

	# Add rust components/targets
	rustup component add rustfmt
	rustup component add clippy
	rustup target add wasm32-unknown-unknown

	# Install apt packages
	sudo apt install -y nginx

	$(MAKE) ${DB_FILE}

.PHONY: clean
clean: ## Clean up
	cargo clean
	rm -rf ${BASE_DIR}
	rm -rf dist
	rm -f cobertura.xml

.PHONY: help
help: ## Show this help
	$(eval HELP_COL_WIDTH:=15)
	@echo "Makefile targets:"
	@grep -E '[^\s]+:.*?## .*$$' ${MAKEFILE_LIST} | grep -v grep | envsubst | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-${HELP_COL_WIDTH}s\033[0m %s\n", $$1, $$2}'
