.PHONY: setup
setup: ${DEV_MEDIA_DIR} ${DEV_DB_FILE} ## Setup project dependencies and dirs
	$(info $(M) Installing cargo binaries)
	$(Q) cargo install watchexec-cli
	$(Q) cargo install trunk

	$(info $(M) Add rust components/targets)
	$(Q) rustup component add rustfmt
	$(Q) rustup component add clippy
	$(Q) rustup target add wasm32-unknown-unknown

	$(info $(M) Install apt packages)
	$(Q) sudo apt install -y nginx ffmpeg

	$(info $(M) Setup done!)

.PHONY: scp-img
scp-img:
	$(info $(M) Copying images)
	$(Q) scp -r stufur:media ${DEV_BASE_DIR}
	$(info $(M) Copy done!)
