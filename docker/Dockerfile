########################
# Step 1:  Build SPIS  #
########################
FROM rust:1.75.0-bookworm as builder

# Setup environment
ENV PROJECT_DIR=/project
ENV DATABASE_URL=sqlite:$PROJECT_DIR/data/spis.db
ENV SPIS_BIN=/spis

# Copy in source
COPY . $PROJECT_DIR
WORKDIR $PROJECT_DIR

# Build SPIS
RUN --mount=type=cache,target=$PROJECT_DIR/target \
    --mount=type=cache,target=$CARGO_HOME/registry \
    cargo build --release --no-default-features -F release && \
    cp $PROJECT_DIR/target/release/spis $SPIS_BIN

########################
# Step 2: Create Image #
########################
FROM nginx:1.25.3-bookworm

# Add extra binaries
RUN apt-get update && \
    apt-get install -y \
      supervisor \
      ffmpeg \
    && rm -rf /var/lib/apt/lists/*

# Copy custom config
COPY docker/40-user-fix.sh /docker-entrypoint.d/40-user-fix.sh
COPY docker/nginx.conf /etc/nginx/templates/default.conf.template
COPY docker/supervisord.conf /etc/supervisor/supervisord.conf

# Set SPIS default variables
ENV \
    NGINX_PORT=8080 \
    RUST_LOG=error,spis=info \
    SPIS_MEDIA_DIR=/var/lib/spis/media \
    SPIS_DATA_DIR=/var/lib/spis/data \
    SPIS_API_MEDIA_PATH=/assets/media \
    SPIS_API_THUMBNAIL_PATH=/assets/thumbnails \
    SPIS_SERVER_SOCKET=/var/run/spis.sock

# Copy binary
COPY --from=builder /spis /usr/bin/spis

# Validate binary and create directories
RUN /usr/bin/spis --version && \
    mkdir -p ${SPIS_MEDIA_DIR} ${SPIS_DATA_DIR}

CMD ["/usr/bin/supervisord", "-c", "/etc/supervisor/supervisord.conf"]
