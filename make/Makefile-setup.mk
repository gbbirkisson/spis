.PHONY: setup-toolchain
setup-toolchain: ## Setup rust toolchain
	$(info $(M) Setup rust toolchain)
	$(Q) rustup show
	$(Q) cat rust-toolchain.toml | grep '# bin' | xargs -n 4 sh -c 'cargo install --version $$3 $$2'

.PHONY: setup
setup: ${DEV_MEDIA_DIR} ${DEV_DB_FILE} setup-toolchain ## Setup project dependencies and dirs
	$(Q) echo "$(M) Installing cargo binaries"
	$(Q) cargo install watchexec-cli

	$(Q) echo "$(M) Install apt packages"
	$(Q) sudo apt install -y nginx ffmpeg

	$(Q) echo "$(M) Setup done!"

.PHONY: scp-img
scp-img:
	$(info $(M) Copying images)
	$(Q) scp -r stufur:media ${DEV_BASE_DIR}
	$(Q) echo "$(M) Copy done!"
