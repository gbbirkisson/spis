DEV_NGINX_CONF_TEMPLATE:=dev/nginx.conf
DEV_NGINX_CONF:=/tmp/nginx.conf

${DEV_MEDIA_DIR}:
	$(info $(M) Creating media dir)
	$(Q) mkdir -p ${DEV_MEDIA_DIR}

${DEV_DB_FILE}:
	$(info $(M) Creating state dir)
	$(Q) mkdir -p ${DEV_STATE_DIR}  

	$(info $(M) Creating database)
	$(Q) sqlx --version > /dev/null || cargo install sqlx-cli
	$(Q) sqlx database create

	$(info $(M) Running database migrations)
	$(Q) sqlx migrate run --source spis-server/migrations

.PHONY: dev
dev: ## Run all dev processes
	$(Q) xdg-open http://localhost:7000
	$(Q) $(MAKE) --no-print-directory -j 3 dev-nginx dev-server dev-gui

.PHONY: _dev-run-nginx-nowatch
_dev-run-nginx-nowatch:
	$(Q) cat $(DEV_NGINX_CONF_TEMPLATE) | DOLLAR='$$' envsubst > $(DEV_NGINX_CONF)
	$(Q) nginx -c ${DEV_NGINX_CONF}

.PHONY: dev-nginx
dev-nginx: ## Run nginx
	$(info $(M) Running nginx dev)
	$(Q) watchexec -r -w dev/nginx.conf -- make _dev-run-nginx-nowatch

.PHONY: dev-server
dev-server: ${DEV_MEDIA_DIR} ${DEV_DB_FILE} ## Run server
	$(info $(M) Running server dev)
	$(Q) watchexec -r -e rs,toml -w spis-model -w spis-server -- cargo run -p spis-server

TEST_FILE?=somefile.mp4
.PHONY: dev-processing
dev-processing: ${DEV_MEDIA_DIR} ${DEV_DB_FILE} ## Run processing of file
	$(info $(M) Running processing dev)
	$(Q) watchexec -r -e rs,toml -w spis-server -- cargo run -p spis-server -- \
		-t "dev/api/media/${TEST_FILE}" 

.PHONY: dev-gui
dev-gui: ## Run gui
	$(info $(M) Running gui dev)
	$(Q) cd spis-gui && trunk serve --port 9000

.PHONY: dev-check-server
dev-check-server: ${DEV_DB_FILE} ## Run check on server
	$(info $(M) Running server checks dev)
	# $(Q) watchexec -r -e rs,toml -w spis-model -w spis-server -- cargo check -p spis-server
	$(Q) watchexec -r -e rs,toml -w spis-model -w spis-server -- cargo clippy -p spis-server -- -W clippy::pedantic -W clippy::nursery -W clippy::unwrap_used

TEST_PROJ?=spis-server
TEST_NAME?=
.PHONY: dev-test
dev-test: ## Run specific test
	$(info $(M) Running test dev)
	$(Q) test ${TEST_NAME} || (echo "Set env var TEST_NAME to specify test name!"; exit 1)
	$(Q) watchexec -r -e rs,toml -w ${TEST_PROJ} -- cargo test -q -p ${TEST_PROJ} ${TEST_NAME} -- --nocapture
