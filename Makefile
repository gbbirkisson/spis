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
	ln -sf AGENTS.md CLAUDE.md
	ln -sf AGENTS.md GEMINI.md
	rustup show
	cargo install --locked sqlx-cli@0.8.0
	cargo install --locked cargo-tarpaulin@0.31.0
	cargo install --locked watchexec-cli

.PHONY: dev-clippy
dev-clippy: ${DATABASE}
	watchexec --stop-timeout=0 -r -e rs,toml,html,css -- \
		cargo clippy -F dev -- --no-deps -D warnings

.PHONY: dev-spis
dev-spis: ${DATABASE} ${MEDIA_DIR} ${THUMBNAIL_DIR}
	watchexec --stop-timeout=0 -r -e rs,toml,html,css -- \
		cargo run --color always -F dev

.PHONY: dev-nginx
dev-nginx: ${DATABASE} ${MEDIA_DIR} ${THUMBNAIL_DIR}
	bash -c 'RUST_LOG=error cargo run -q -- template nginx --full > /tmp/nginx.conf && nginx -g "daemon off;" -c /tmp/nginx.conf'

.PHONY: dev
dev:
	$(MAKE) --no-print-directory -j 2 dev-nginx dev-spis

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

.PHONY: template
template: ${DATABASE} ${MEDIA_DIR} ${THUMBNAIL_DIR}
	cargo build --color always
	cargo run -q -- \
		--server-socket /storage/spis/data/spis.sock \
		--data-dir /storage/spis/data \
		--media-dir /storage/spis/media template \
		nginx --port 8080 > examples/systemd/nginx.conf
	cargo run -q -- \
		--server-socket /storage/spis/data/spis.sock \
		--data-dir /storage/spis/data \
		--media-dir /storage/spis/media template \
		systemd --bin /usr/bin/spis --user www-data > examples/systemd/spis.service
	cargo run -q -- \
		--data-dir /tmp/spis_data \
		--media-dir /tmp/spis_media template \
		docker-compose > examples/docker/docker-compose.yml
	git diff --exit-code

.PHONY: docker-build
docker-build: release/spis-${X86_64}
	docker build -t spis-local -f docker/Dockerfile .

.PHONY: docker-exec
docker-exec: docker-build
	docker run -it --rm \
		--entrypoint bash \
		spis-local

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
