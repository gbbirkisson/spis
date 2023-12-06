RELEASE_CMD:=cross

RELEASE_GUI:=spis-gui/dist
RELEASE_X86_60_GNU:=release/spis-server-x86_64-unknown-linux-gnu
RELEASE_ARMV7_GNUEABIHF:=release/spis-server-armv7-unknown-linux-gnueabihf
RELEASE_AARCH64_GNU:=release/spis-server-aarch64-unknown-linux-gnu

${RELEASE_GUI}:
	$(info $(M) Build GUI release)
	$(Q) cd spis-gui && trunk build --release
	$(Q) rm -r target/release

${RELEASE_X86_60_GNU}: ${RELEASE_GUI} ${DEV_DB_FILE}
	$(info $(M) Build x86 release)
	$(Q) $(RELEASE_CMD) build -p spis-server --features release --release --target x86_64-unknown-linux-gnu
	$(Q) mkdir -p release
	$(Q) cp target/x86_64-unknown-linux-gnu/release/spis-server ${RELEASE_X86_60_GNU}

${RELEASE_ARMV7_GNUEABIHF}: ${RELEASE_GUI} ${DEV_DB_FILE}
	$(info $(M) Build ARMv7 release)
	$(Q) $(RELEASE_CMD) build -p spis-server --features release --release --target armv7-unknown-linux-gnueabihf
	$(Q) mkdir -p release
	$(Q) cp target/armv7-unknown-linux-gnueabihf/release/spis-server ${RELEASE_ARMV7_GNUEABIHF}

${RELEASE_AARCH64_GNU}: ${RELEASE_GUI} ${DEV_DB_FILE}
	$(info $(M) Build AARCH64 release)
	$(Q) $(RELEASE_CMD) build -p spis-server --features release --release --target aarch64-unknown-linux-gnu
	$(Q) mkdir -p release
	$(Q) cp target/aarch64-unknown-linux-gnu/release/spis-server ${RELEASE_AARCH64_GNU}

.PHONY: release
release: ${RELEASE_X86_60_GNU} ${RELEASE_ARMV7_GNUEABIHF} ${RELEASE_AARCH64_GNU} ## Create release
	$(info $(M) Release created!)

.PHONY: release-gui
release-gui: ${RELEASE_GUI} ## Create release build of GUI
	$(info $(M) Release for gui created!)

.PHONY: docker-local
docker-local: ${RELEASE_X86_60_GNU} ## Create and run docker image
	$(info $(M) Building docker image)
	$(Q) docker build -t spis-local -f docker/Dockerfile .

	$(Q) echo "$(M) Running docker image"
	$(Q) docker run -it --rm \
		-p 8080:8080 \
		-v ${PWD}/dev/api/media:/var/lib/spis/media \
		-e SPIS_PROCESSING_RUN_ON_START=true \
		spis-local
