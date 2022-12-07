.DEFAULT_GOAL:=help

DEV_BASE_DIR:=dev/api
DEV_MEDIA_DIR:=${DEV_BASE_DIR}/media
DEV_STATE_DIR:=${DEV_BASE_DIR}/state
DEV_DB_FILE:=${DEV_STATE_DIR}/spis.db

include Makefile-ci.mk
include Makefile-dev.mk
include Makefile-setup.mk
include Makefile-release.mk

.PHONY: help
help: ## Show this help
	$(eval HELP_COL_WIDTH:=15)
	@echo "Makefile targets:"
	@grep -E '[^\s]+:.*?## .*$$' ${MAKEFILE_LIST} | sed 's/Makefile://g' | sed 's/.*\.mk://g' | grep -v grep | envsubst | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-${HELP_COL_WIDTH}s\033[0m %s\n", $$1, $$2}'

.PHONY: clean-state
clean-state: ## Clean up local state
	rm -rf ${DEV_STATE_DIR}

.PHONY: clean
clean: clean-state ## Clean up
	cargo clean
	rm -rf spis-gui/dist
	rm -rf release
	rm -f cobertura.xml
