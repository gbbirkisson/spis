RELEASE_GUI:=spis-gui/dist
RELEASE_X86_60_GNU:=release/spis-server-x86_64-unknown-linux-gnu
RELEASE_ARMV7_GNUEABIHF:=release/spis-server-armv7-unknown-linux-gnueabihf

${RELEASE_GUI}:
	$(info $(M) Build GUI release)
	$(Q) trunk --version > /dev/null || cargo install --locked trunk
	$(Q) cd spis-gui && trunk build --release
	$(Q) cp -f spis-gui/manifest.json spis-gui/dist/manifest.json
	$(Q) cp -f assets/logo.png spis-gui/dist/logo.png
	$(Q) rm -r target/release

${RELEASE_X86_60_GNU}: ${RELEASE_GUI} ${DEV_DB_FILE}
	$(info $(M) Build x86 release)
	$(Q) cross build -p spis-server --features release --release --target x86_64-unknown-linux-gnu
	$(Q) mkdir -p release
	$(Q) cp target/x86_64-unknown-linux-gnu/release/spis-server ${RELEASE_X86_60_GNU}

${RELEASE_ARMV7_GNUEABIHF}: ${RELEASE_GUI} ${DEV_DB_FILE}
	$(info $(M) Build ARM release)
	$(Q) cross build -p spis-server --features release --release --target armv7-unknown-linux-gnueabihf
	$(Q) mkdir -p release
	$(Q) cp target/armv7-unknown-linux-gnueabihf/release/spis-server ${RELEASE_ARMV7_GNUEABIHF}

.PHONY: release
release: ${RELEASE_X86_60_GNU} ${RELEASE_ARMV7_GNUEABIHF} ## Create release
	$(info $(M) Release created!)

.PHONY: release-gui
release-gui: ${RELEASE_GUI} ## Create release build of GUI
	$(info $(M) Release for gui created!)

.PHONY: validate
validate:
	$(info $(M) Validate gui version)
	$(Q) test "${GITHUB_REF_NAME}" = "$(shell awk -F ' = ' '$$1 ~ /version/ { gsub(/[\"]/, "", $$2); printf("v%s",$$2) }' spis-gui/Cargo.toml)"

	$(info $(M) Validate model version)
	$(Q) test "${GITHUB_REF_NAME}" = "$(shell awk -F ' = ' '$$1 ~ /version/ { gsub(/[\"]/, "", $$2); printf("v%s",$$2) }' spis-model/Cargo.toml)"

	$(info $(M) Validate server version)
	$(Q) test "${GITHUB_REF_NAME}" = "$(shell awk -F ' = ' '$$1 ~ /version/ { gsub(/[\"]/, "", $$2); printf("v%s",$$2) }' spis-server/Cargo.toml)"

	$(info $(M) Validation passed!)

.PHONY: docker-local
docker-local: ${RELEASE_X86_60_GNU} ## Create and run docker image
	$(info $(M) Building docker image)
	$(Q) docker build -t spis-local -f docker/Dockerfile .

	$(info $(M) Running docker image)
	$(Q) docker run -it --rm \
		-p 8080:8080 \
		-v ${PWD}/dev/api/media:/var/lib/spis/media \
		-e SPIS_PROCESSING_RUN_ON_START=true \
		spis-local
