FROM nginx:1.23.2 as builder

ENV \
    TARGET_DIR=/target \
    BIN_LOCATION=/usr/bin/spis-server

COPY target ${TARGET_DIR}

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
    RUST_LOG=error,spis_server=info \
    SPIS_SERVER.SOCKET=/tmp/spis.socket \
    SPIS_PROCESSING.RUN_ON_START=true \
    SPIS_MEDIA_DIR=/var/lib/spis/media \
    SPIS_DATA_DIR=/var/lib/spis/data

# Set nginx variables
ENV \
    NGINX_PORT=8080 \
    NGINX_SOCKET=/tmp/spis.socket

# Copy default nginx configuration
COPY --from=builder /etc/nginx /etc/nginx
COPY --from=builder /docker-entrypoint.sh /docker-entrypoint.sh
COPY --from=builder /docker-entrypoint.d /docker-entrypoint.d

# Copy in custom configuration
COPY docker/40-user-fix.sh /docker-entrypoint.d/40-user-fix.sh
COPY docker/nginx.conf /etc/nginx/templates/default.conf.template
COPY docker/supervisord.conf /etc/supervisor/supervisord.conf

# Setup SPIS
COPY --from=builder /usr/bin/spis-server /usr/bin/spis-server
RUN mkdir -p ${SPIS_MEDIA_DIR} ${SPIS_DATA_DIR}

CMD ["/usr/bin/supervisord", "-c", "/etc/supervisor/supervisord.conf"]