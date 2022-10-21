.DEFAULT_GOAL:=help

NGINX_CONF:=dev/nginx.conf
NGINX_CONF_TMP:=/tmp/nginx.conf

.PHONY: dev-run-nginx-nowatch
dev-run-nginx-nowatch: ## Run nginx
	cat $(NGINX_CONF) | envsubst > $(NGINX_CONF_TMP)
	nginx -c $(NGINX_CONF_TMP)

.PHONY: dev-run-nginx
dev-run-nginx: ## Run nginx and reload on config change
	watchexec -w dev/nginx.conf -r -- make dev-run-nginx-nowatch

.PHONY: help
help: ## Show this help
	$(eval HELP_COL_WIDTH:=22)
	@echo "Makefile targets:"
	@grep -E '[^\s]+:.*?## .*$$' ${MAKEFILE_LIST} | grep -v grep | envsubst | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-${HELP_COL_WIDTH}s\033[0m %s\n", $$1, $$2}'
