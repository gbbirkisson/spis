DEV_NGINX_CONF_TEMPLATE:=dev/nginx.conf
DEV_NGINX_CONF:=/tmp/nginx.conf

${DEV_MEDIA_DIR}:
	mkdir -p ${DEV_MEDIA_DIR}

${DEV_DB_FILE}:
	mkdir -p ${DEV_STATE_DIR}  
	sqlx --version > /dev/null || cargo install sqlx-cli
	sqlx database create
	sqlx migrate run --source spis-server/migrations

.PHONY: dev
dev: ## Run all dev processes
	$(MAKE) --no-print-directory -j 3 dev-nginx dev-server dev-gui
	sleep 3
	xdg-open http://localhost:7000

.PHONY: _dev-run-nginx-nowatch
_dev-run-nginx-nowatch:
	cat $(DEV_NGINX_CONF_TEMPLATE) | DOLLAR='$$' envsubst > $(DEV_NGINX_CONF)
	nginx -c ${DEV_NGINX_CONF}

.PHONY: dev-nginx
dev-nginx: ## Run nginx
	watchexec -r -w dev/nginx.conf -- make _dev-run-nginx-nowatch

.PHONY: dev-server
dev-server: ${DEV_MEDIA_DIR} ${DEV_DB_FILE} ## Run the server
	watchexec -r -e rs,toml -w spis-model -w spis-server -- cargo run -p spis-server

TEST_FILE?=somefile.mp4
.PHONY: dev-processing
dev-processing: ${DEV_MEDIA_DIR} ${DEV_DB_FILE} ## Run processing of file
	watchexec -r -e rs,toml -w spis-server -- cargo run -p spis-server -- \
		-t "dev/api/media/${TEST_FILE}" 


.PHONY: dev-gui
dev-gui: ## Run the gui
	cd spis-gui && trunk serve --port 9000

.PHONY: dev-check-server
dev-check-server: ${DEV_DB_FILE} ## Run check continuously on server
	watchexec -r -e rs,toml -w spis-model -w spis-server -- cargo check -p spis-server

TEST_PROJ?=spis-server
TEST_NAME?=
.PHONY: dev-test
dev-test: ## Run specific test
	test ${TEST_NAME} || (echo "Set env var TEST_NAME to specify test name!"; exit 1)
	watchexec -r -e rs,toml -w ${TEST_PROJ} -- cargo test -q -p ${TEST_PROJ} ${TEST_NAME} -- --nocapture
