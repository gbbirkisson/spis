RELEASE_GUI:=spis-gui/dist
RELEASE_NATIVE_SERVER:=spis-server/target/release/spis-server

${RELEASE_GUI}:
	cd spis-gui && trunk build --release
	cp -f spis-gui/manifest.json spis-gui/dist/manifest.json
	cp -f logo.png spis-gui/dist/logo.png

${RELEASE_NATIVE_SERVER}: ${RELEASE_GUI} ${DEV_DB_FILE}
	cargo build -p spis-server --features release --release

.PHONY: docker-local
docker-local: ${RELEASE_NATIVE_SERVER} ## Run docker image
	docker build -t spis-local -f docker/Dockerfile .
	docker run -it --rm -p 8080:8080 -v ${PWD}/dev/api/media:/var/lib/spis/media spis-local

#	cargo install cross --git https://github.com/cross-rs/cross
#cross build -p spis-server --features release --release --target armv7-unknown-linux-gnueabihf