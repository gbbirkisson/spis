# SPIS (Simple Private Image Server)

## Project Overview

**SPIS** is a lightweight, high-performance media server designed to host private collections of
images and videos. It emphasizes simplicity, speed, and privacy.

*   **Core Technology:** Rust (Axum, Tokio, Sqlx).
*   **Database:** SQLite (embedded).
*   **Frontend:** Server-side rendered HTML with HTMX for dynamic interactions.
*   **Media Serving:** Nginx is used as a reverse proxy to efficiently serve static media files
and thumbnails, while the `spis` binary handles application logic and API requests.

## Architecture

The system consists of two main running components:
1.  **`spis` Binary:**
    *   **File Watcher/Walker:** Scans the media directory, processes new files (extracts EXIF
    data, generates thumbnails), and updates the SQLite database.
    *   **Web Server:** Handles user interface (HTMX), metadata queries, and API endpoints.
2.  **Nginx:**
    *   Acts as the front-facing web server.
    *   Serves the static media files (images, videos) and generated thumbnails directly from
    the filesystem for maximum performance.
    *   Proxies application requests to the `spis` binary.

## Key Components & Code Structure

### `src/`

*   **`main.rs`**: Entry point. Handles CLI argument parsing (`clap`), config loading, and orchestrates the startup of the database, pipeline, and web server.
*   **`lib.rs`**: Shared library definitions and module exports.
*   **`pipeline.rs`**: The core media ingestion engine.
    *   **Watcher:** Listens for file system events.
    *   **Walker:** Scans directories on startup or schedule.
    *   **Processor:** Uses `rayon` for parallel processing of images/videos (metadata extraction, thumbnail generation).
*   **`db.rs`**: Database interactions using `sqlx`.
*   **`server/`**: Axum server implementation.
    *   **`mod.rs`**: Server configuration and routing.
    *   **`hx/`**: HTMX-based UI components (Gallery, Navigation Bar, Preview).
    *   **`assets.rs`**: Static asset handling for the UI.

### `migrations/`

Contains SQL files for database schema versioning. Managed via `sqlx`.

### `templates/`

*   **`config/`**: Templates for Nginx, Systemd, and Docker Compose configurations.
*   **`web/`**: HTML templates for the UI (Askama).

## Development Workflow

* **After each change run**:

    ```bash
    make lint
    ```
*   **Commit Messages:**
    This project enforces **Conventional Commits** (e.g., `feat: ...`, `fix: ...`).
