.PHONY: dl-img
dl-img: ${DEV_MEDIA_DIR} ## Download 20 random images
	./dev/images.sh 20 ${DEV_MEDIA_DIR}

.PHONY: setup
setup: ${DEV_MEDIA_DIR} ${DEV_DB_FILE} ## Setup project dependencies and dirs
	# Install cargo binaries
	cargo install watchexec-cli
	cargo install trunk

	# Add rust components/targets
	rustup component add rustfmt
	rustup component add clippy
	rustup target add wasm32-unknown-unknown

	# Install apt packages
	sudo apt install -y nginx