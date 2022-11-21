FROM nginx:1.23.2 as nginx

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    supervisor \
    nginx \
    gettext-base \
    && rm -rf /var/lib/apt/lists/*

ENV \
    NGINX_PORT=8080 \
    RUST_LOG=error,spis_server=info \
    SPIS_MEDIA_DIR=/var/lib/spis/media \
    SPIS_DATA_DIR=/var/lib/spis/data

RUN mkdir -p ${SPIS_MEDIA_DIR} ${SPIS_DATA_DIR}

COPY --from=nginx /etc/nginx /etc/nginx
COPY --from=nginx /docker-entrypoint.sh /docker-entrypoint.sh
COPY --from=nginx /docker-entrypoint.d /docker-entrypoint.d

COPY docker/40-user-fix.sh /docker-entrypoint.d/40-user-fix.sh
COPY docker/nginx.conf /etc/nginx/templates/default.conf.template
COPY docker/supervisord.conf /etc/supervisor/supervisord.conf

COPY target/release/spis-server /usr/bin/spis-server

CMD ["/usr/bin/supervisord", "-c", "/etc/supervisor/supervisord.conf"]