.DEFAULT_GOAL:=help

NGINX_CONF:=dev/nginx.conf
NGINX_CONF_TMP:=/tmp/nginx.conf

BASE_DIR:=dev/api
MEDIA_DIR:=${BASE_DIR}/media
STATE_DIR:=${BASE_DIR}/state
THUMBNAIL_DIR:=${STATE_DIR}/thumbnails
DB_FILE:=${STATE_DIR}/spis.db


${MEDIA_DIR}:
	mkdir -p ${MEDIA_DIR}
	$(MAKE) dl-img

${STATE_DIR}:
	mkdir -p ${STATE_DIR}

${THUMBNAIL_DIR}:
	mkdir -p ${THUMBNAIL_DIR}

${DB_FILE}: ${STATE_DIR}
	sqlx --version > /dev/null || cargo install sqlx-cli
	sqlx database create
	sqlx migrate run --source spis-server/migrations

.PHONY: _dev-run-nginx-nowatch
_dev-run-nginx-nowatch:
	cat $(NGINX_CONF) | DOLLAR='$$' envsubst > $(NGINX_CONF_TMP)
	nginx -c $(NGINX_CONF_TMP)

.PHONY: fmt
fmt: ## Run format check
	cargo fmt -- --check

.PHONY: lint
lint: ${DB_FILE} ## Run lint check
	cargo clippy -- -D warnings

.PHONY: test-nocov
test-nocov: ${DB_FILE} ## Run tests with no coverage report
	cargo test

.PHONY: test
test: ${DB_FILE} ## Run tests with coverage report
	cargo tarpaulin --version > /dev/null || cargo install cargo-tarpaulin
	cargo tarpaulin --ignore-tests --all-features --workspace --timeout 120 --skip-clean --out Xml

audit: ## Run audit on dependencies
	cargo audit --version > /dev/null || cargo install cargo-audit
	cargo audit

.PHONY: ci
ci: fmt lint audit test ## Run CI steps

.PHONY: dev
dev: ## Run all dev processes
	x-terminal-emulator -t nginx -e make dev-nginx &
	x-terminal-emulator -t server -e make dev-server &
	x-terminal-emulator -t gui -e make dev-gui &
	xdg-open http://localhost:7000

.PHONY: dev-nginx
dev-nginx: ## Run nginx
	watchexec -r -w dev/nginx.conf -- make _dev-run-nginx-nowatch

.PHONY: dev-server
dev-server: ${MEDIA_DIR} ${THUMBNAIL_DIR} ${DB_FILE} ## Run the server
	watchexec -r -e rs,toml -w spis-model -w spis-server -- cargo run -p spis-server

.PHONY: dev-gui
dev-gui: ## Run the gui
	cd spis-gui && trunk serve --port 9000

TEST_PROJ?=spis-server
TEST_NAME?=
.PHONY: dev-test
dev-test: ## Run specific test
	test ${TEST_NAME} || (echo "Set env var TEST_NAME to specify test name!"; exit 1)
	watchexec -r -e rs,toml -w ${TEST_PROJ} -- cargo test -q -p ${TEST_PROJ} ${TEST_NAME} -- --nocapture

.PHONY: dl-img
dl-img: ${MEDIA_DIR} ## Download 20 random images
	./dev/images.sh 20 ${MEDIA_DIR}

.PHONY: release
release: ## Create release build
	cd spis-gui && trunk build --release
	cp -f spis-gui/manifest.json spis-gui/dist/manifest.json
	cp -f logo.png spis-gui/dist/logo.png
	cargo build -p spis-server --features release --release
	#cross build -p spis-server --features release --release --target armv7-unknown-linux-gnueabihf

.PHONY: docker-build
docker-build:
	docker build -t test -f docker/Dockerfile .

.PHONY: docker-run
docker-run: docker-build
	docker run -it --rm -p 8080:8080 -v ${PWD}/dev/api/media:/var/lib/spis/media test

.PHONY: setup
setup: ${MEDIA_DIR} ${THUMBNAIL_DIR} ${DB_FILE} ## Setup project dependencies and dirs
	# Install cargo binaries
	cargo install watchexec-cli
	cargo install trunk
	cargo install cargo-watch
	cargo install cross --git https://github.com/cross-rs/cross

	# Add rust components/targets
	rustup component add rustfmt
	rustup component add clippy
	rustup target add wasm32-unknown-unknown

	# Install apt packages
	sudo apt install -y nginx

.PHONY: clean
clean: ## Clean up
	cargo clean
	rm -rf ${BASE_DIR}
	rm -rf spis-gui/dist
	rm -f cobertura.xml

.PHONY: help
help: ## Show this help
	$(eval HELP_COL_WIDTH:=15)
	@echo "Makefile targets:"
	@grep -E '[^\s]+:.*?## .*$$' ${MAKEFILE_LIST} | grep -v grep | envsubst | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-${HELP_COL_WIDTH}s\033[0m %s\n", $$1, $$2}'
