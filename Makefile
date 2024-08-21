SOURCES:=$(shell find src -type f -name '*.rs')

X86_64:=x86_64-unknown-linux-gnu
ARMV7:=armv7-unknown-linux-gnueabihf
AARCH64:=aarch64-unknown-linux-gnu

DATABASE:=$(shell echo ${DATABASE_URL} | awk -F':' '{print $$2}')
MEDIA_DIR:=data/media
THUMBNAIL_DIR:=data/thumbnails

${DATABASE}:
	mkdir -p $(shell dirname ${DATABASE})
	sqlx database create
	sqlx migrate run --source migrations

${MEDIA_DIR}:
	mkdir -p ${MEDIA_DIR}

${THUMBNAIL_DIR}:
	mkdir -p ${THUMBNAIL_DIR}

_release: ${DATABASE}
	cargo build --locked --release --target ${TARGET}
	mkdir -p release
	cp target/${TARGET}/release/spis release/spis-${TARGET}

release/spis-${X86_64}: ${SOURCES}
	TARGET=${X86_64} $(MAKE) --no-print-directory _release

release/spis-${ARMV7}: ${SOURCES}
	TARGET=${ARMV7} $(MAKE) --no-print-directory _release

release/spis-${AARCH64}: ${SOURCES}
	TARGET=${AARCH64} $(MAKE) --no-print-directory _release

.PHONY: toolchain
toolchain:
	rustup show

.PHONY: dev-clippy
dev-clippy: ${DATABASE}
	watchexec --stop-timeout=0 -r -e rs,toml,html,css -- \
		cargo clippy -F dev -- --no-deps -D warnings

.PHONY: dev-spis
dev-spis: ${DATABASE} ${MEDIA_DIR} ${THUMBNAIL_DIR}
	watchexec --stop-timeout=0 -r -e rs,toml,html,css -- \
		cargo run --color always -F dev

.PHONY: lint-fmt
lint-fmt:
	cargo fmt -- --check

.PHONY: lint-clippy
lint-clippy: ${DATABASE}
	cargo clippy -F dev -- --no-deps -D warnings

.PHONY: lint
lint: lint-fmt lint-clippy

.PHONY: test
test: ${DATABASE} ${MEDIA_DIR} ${THUMBNAIL_DIR}
	cargo tarpaulin --ignore-tests --color always --timeout 120 --skip-clean --out Xml

.PHONY: docker-build
docker-build: release/spis-${X86_64}
	docker build -t spis-local -f docker/Dockerfile .

.PHONY: docker-run
docker-run: docker-build
	docker run -it --rm \
		-p 8080:8080 \
		-v ${PWD}/data/media:/var/lib/spis/media \
		-e SPIS_PROCESSING_RUN_ON_START=true \
		spis-local

.PHONY: clean
clean:
	rm -rf target release data/spis.db data/thumbnails
	cargo clean
