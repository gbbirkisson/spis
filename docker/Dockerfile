FROM nginx:1.29.4 AS builder

ENV \
    RELEASE_DIR=/tmp/release \
    BIN_LOCATION=/usr/bin/spis

COPY release ${RELEASE_DIR}

COPY docker/pick-release.sh /pick-release.sh
RUN /pick-release.sh

FROM nginx:1.29.4

RUN apt-get update && apt-get install -y \
    supervisor \
    ffmpeg \
    && rm -rf /var/lib/apt/lists/*

# Set spis variables
ENV \
    NGINX_PORT=8080 \
    RUST_LOG=error,spis=info \
    SPIS_MEDIA_DIR=/var/lib/spis/media \
    SPIS_DATA_DIR=/var/lib/spis/data \
    SPIS_API_MEDIA_PATH=/assets/media \
    SPIS_API_THUMBNAIL_PATH=/assets/thumbnails
ENV \
    SPIS_SERVER_SOCKET=${SPIS_DATA_DIR}/spis.sock

# Copy in custom configuration
COPY docker/docker-entrypoint.d/* /docker-entrypoint.d/
COPY docker/supervisor/* /etc/supervisor/

# Setup SPIS and smoketest
COPY --from=builder /usr/bin/spis /usr/bin/spis
RUN chmod +x /usr/bin/spis && \
    /usr/bin/spis --version && \
    mkdir -p ${SPIS_MEDIA_DIR} ${SPIS_DATA_DIR}

CMD ["supervisord", "-c", "/etc/supervisor/supervisord.conf"]
