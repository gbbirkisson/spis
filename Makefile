.DEFAULT_GOAL:=help

NGINX_CONF:=dev/nginx.conf
NGINX_CONF_TMP:=/tmp/nginx.conf

.PHONY: _dev-run-nginx-nowatch
_dev-run-nginx-nowatch:
	cat $(NGINX_CONF) | envsubst > $(NGINX_CONF_TMP)
	nginx -c $(NGINX_CONF_TMP)

.PHONY: dev-nginx
dev-nginx: ## Run nginx
	watchexec -r -w dev/nginx.conf -- make _dev-run-nginx-nowatch

.PHONY: dev-server
dev-server: ## Run the server
	watchexec -r -e rs -w model -w server -- cargo run -p server

.PHONY: dev-gui
dev-gui: ## Run the gui
	cd gui && trunk serve --port 9000 --proxy-backend http://localhost:7000/api/

.PHONY: help
help: ## Show this help
	$(eval HELP_COL_WIDTH:=15)
	@echo "Makefile targets:"
	@grep -E '[^\s]+:.*?## .*$$' ${MAKEFILE_LIST} | grep -v grep | envsubst | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-${HELP_COL_WIDTH}s\033[0m %s\n", $$1, $$2}'
