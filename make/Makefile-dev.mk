DEV_NGINX_CONF_TEMPLATE:=dev/nginx.conf
DEV_NGINX_CONF:=/tmp/nginx.conf

${DEV_MEDIA_DIR}:
	$(Q) echo "$(M) Creating media dir"
	$(Q) mkdir -p ${DEV_MEDIA_DIR}

${DEV_DB_FILE}:
	$(Q) echo "$(M) Creating state dir"
	$(Q) mkdir -p ${DEV_STATE_DIR}

	$(Q) echo "$(M) Creating database"
	$(Q) sqlx database create

	$(Q) echo "$(M) Running database migrations"
	$(Q) sqlx migrate run --source spis-server/migrations

.PHONY: dev
dev: ## Run all dev processes
	$(Q) xdg-open http://localhost:7000
	$(Q) $(MAKE) --no-print-directory -j 2 dev-nginx dev-server

.PHONY: _dev-run-nginx-nowatch
_dev-run-nginx-nowatch:
	$(Q) cat $(DEV_NGINX_CONF_TEMPLATE) | DOLLAR='$$' envsubst > $(DEV_NGINX_CONF)
	$(Q) nginx -c ${DEV_NGINX_CONF}

.PHONY: dev-nginx
dev-nginx: ## Run nginx
	$(Q) echo "$(M) Running nginx dev"
	$(Q) watchexec --stop-timeout=0 -r -w dev/nginx.conf -- make _dev-run-nginx-nowatch

.PHONY: dev-server
dev-server: ${DEV_MEDIA_DIR} ${DEV_DB_FILE} ## Run server
	$(Q) echo "$(M) Running server dev"
	$(Q) watchexec --stop-timeout=0 -r -e rs,toml -- \
		cargo run

TEST_FILE?=somefile.mp4
.PHONY: dev-processing
dev-processing: ${DEV_MEDIA_DIR} ${DEV_DB_FILE} ## Run processing of file
	$(Q) echo "$(M) Running processing dev"
	$(Q) watchexec --stop-timeout=0 -r -e rs,toml -- \
		cargo run -- -t "dev/api/media/${TEST_FILE}"

.PHONY: dev-check-server
dev-check-server: ${DEV_DB_FILE} ## Run check on server
	$(Q) echo "$(M) Running server checks dev"
	$(Q) watchexec --stop-timeout=0 -r -e rs,toml -- \
		cargo clippy -- -W clippy::pedantic -W clippy::nursery -W clippy::unwrap_used

TEST_NAME?=
.PHONY: dev-test
dev-test: ## Run specific test
	$(Q) echo "$(M) Running test dev"
	$(Q) test ${TEST_NAME} || (echo "Set env var TEST_NAME to specify test name!"; exit 1)
	$(Q) watchexec --stop-timeout=0 -r -e rs,toml -- \
		cargo test -q ${TEST_NAME} -- --nocapture
