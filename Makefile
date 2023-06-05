.DEFAULT_GOAL:=help

Q = $(if $(filter 1,$V),,@)
M = $(shell printf "\033[34;1m▶\033[0m")

DEV_BASE_DIR:=dev/api
DEV_MEDIA_DIR:=${DEV_BASE_DIR}/media
DEV_STATE_DIR:=${DEV_BASE_DIR}/state
DEV_DB_FILE:=${DEV_STATE_DIR}/spis.db

include make/Makefile-setup.mk
include make/Makefile-ci.mk
include make/Makefile-dev.mk
include make/Makefile-release.mk

.PHONY: help
help: ## Show this help
	$(info $(M) Makefile targets:)
	$(eval HELP_COL_WIDTH:=16)
	$(Q) grep -E '[^\s]+:.*?## .*$$' ${MAKEFILE_LIST} | sed 's/Makefile://g' | sed 's/.*\.mk://g' | grep -v grep | envsubst | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-${HELP_COL_WIDTH}s\033[0m %s\n", $$1, $$2}'

.PHONY: clean
clean: clean-state ## Clean up everything
	$(info $(M) Cleaning everything)
	$(Q) cargo clean
	$(Q) rm -rf spis-gui/dist
	$(Q) rm -rf release
	$(Q) rm -f cobertura.xml

.PHONY: clean-state
clean-state: ## Clean up local state
	$(info $(M) Clean up local state)
	$(Q) rm -rf ${DEV_STATE_DIR}

