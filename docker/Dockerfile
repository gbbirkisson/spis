FROM nginx:1.23.2 as builder

ENV \
    RELEASE_DIR=/tmp/release \
    BIN_LOCATION=/usr/bin/spis-server

COPY release ${RELEASE_DIR}

COPY docker/pick-release.sh /pick-release.sh
RUN /pick-release.sh

FROM debian:bookworm-slim

# Install dependencies
RUN apt-get update && apt-get install -y \
    supervisor \
    nginx \
    gettext-base \
    && rm -rf /var/lib/apt/lists/*

# Set spis variables
ENV \
    NGINX_PORT=8080 \
    RUST_LOG=error,spis_server=info \
    SPIS_MEDIA_DIR=/var/lib/spis/media \
    SPIS_DATA_DIR=/var/lib/spis/data \
    SPIS_API_MEDIA_PATH=/assets/media \
    SPIS_API_THUMBNAIL_PATH=/assets/thumbnails \
    SPIS_SERVER_SOCKET=/var/run/spis.sock

# Copy default nginx configuration
COPY --from=builder /etc/nginx /etc/nginx
COPY --from=builder /docker-entrypoint.sh /docker-entrypoint.sh
COPY --from=builder /docker-entrypoint.d /docker-entrypoint.d

# Copy in custom configuration
COPY docker/40-user-fix.sh /docker-entrypoint.d/40-user-fix.sh
COPY docker/nginx.conf /etc/nginx/templates/default.conf.template
COPY docker/supervisord.conf /etc/supervisor/supervisord.conf

# Setup SPIS and smoketest
COPY --from=builder /usr/bin/spis-server /usr/bin/spis-server
RUN chmod +x /usr/bin/spis-server && \
    /usr/bin/spis-server --version && \
    mkdir -p ${SPIS_MEDIA_DIR} ${SPIS_DATA_DIR}

CMD ["/usr/bin/supervisord", "-c", "/etc/supervisor/supervisord.conf"]