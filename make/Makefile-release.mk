RELEASE_CMD:=cross

RELEASE_X86_60_GNU:=release/spis-x86_64-unknown-linux-gnu
RELEASE_ARMV7_GNUEABIHF:=release/spis-armv7-unknown-linux-gnueabihf
RELEASE_AARCH64_GNU:=release/spis-aarch64-unknown-linux-gnu

${RELEASE_X86_60_GNU}: ${DEV_DB_FILE}
	$(Q) echo "$(M) Build x86 release"
	$(Q) $(RELEASE_CMD) build --features release --release --target x86_64-unknown-linux-gnu
	$(Q) mkdir -p release
	$(Q) cp target/x86_64-unknown-linux-gnu/release/spis ${RELEASE_X86_60_GNU}

${RELEASE_ARMV7_GNUEABIHF}: ${DEV_DB_FILE}
	$(Q) echo "$(M) Build ARMv7 release"
	$(Q) $(RELEASE_CMD) build --features release --release --target armv7-unknown-linux-gnueabihf
	$(Q) mkdir -p release
	$(Q) cp target/armv7-unknown-linux-gnueabihf/release/spis ${RELEASE_ARMV7_GNUEABIHF}

${RELEASE_AARCH64_GNU}: ${RELEASE_GUI} ${DEV_DB_FILE}
	$(Q) echo "$(M) Build AARCH64 release"
	$(Q) $(RELEASE_CMD) build --features release --release --target aarch64-unknown-linux-gnu
	$(Q) mkdir -p release
	$(Q) cp target/aarch64-unknown-linux-gnu/release/spis ${RELEASE_AARCH64_GNU}

.PHONY: release
release: ${RELEASE_X86_60_GNU} ${RELEASE_ARMV7_GNUEABIHF} ${RELEASE_AARCH64_GNU} ## Create release
	$(Q) echo "$(M) Release created!"

.PHONY: docker-local
docker-local: ${RELEASE_X86_60_GNU} ## Create and run docker image
	$(Q) echo "$(M) Building docker image"
	$(Q) docker build -t spis-local -f docker/Dockerfile .

	$(Q) echo "$(M) Running docker image"
	$(Q) docker run -it --rm \
		-p 8080:8080 \
		-v ${PWD}/dev/api/media:/var/lib/spis/media \
		-e SPIS_PROCESSING_RUN_ON_START=true \
		spis-local
