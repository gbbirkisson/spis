RELEASE_GUI:=spis-gui/dist
RELEASE_NATIVE_SERVER:=spis-server/target/release/spis-server
RELEASE_X86_60_GNU:=target/x86_64-unknown-linux-gnu/release/spis-server
RELEASE_ARMV7_GNUEABIHF:=target/armv7-unknown-linux-gnueabihf/release/spis-server

${RELEASE_GUI}:
	cd spis-gui && trunk build --release
	cp -f spis-gui/manifest.json spis-gui/dist/manifest.json
	cp -f logo.png spis-gui/dist/logo.png
	rm -r target/release

${RELEASE_X86_60_GNU}: ${RELEASE_GUI} ${DEV_DB_FILE}
	cross build -p spis-server --features release --release --target x86_64-unknown-linux-gnu

${RELEASE_ARMV7_GNUEABIHF}: ${RELEASE_GUI} ${DEV_DB_FILE}
	cross build -p spis-server --features release --release --target armv7-unknown-linux-gnueabihf

.PHONY: release-gui
release-gui: ${RELEASE_GUI} ## Create release build of GUI

.PHONY: release ## Build release assets
release: ${RELEASE_X86_60_GNU} ${RELEASE_ARMV7_GNUEABIHF}

.PHONY: docker-local
docker-local: ${RELEASE_X86_60_GNU} ## Run docker image
	docker build -t spis-local -f docker/Dockerfile .
	docker run -it --rm -p 8080:8080 -v ${PWD}/dev/api/media:/var/lib/spis/media spis-local
