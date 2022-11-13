CREATE TABLE IF NOT EXISTS images (
    id          UUID    PRIMARY KEY NOT NULL,
    image       TEXT                NOT NULL, 
    created_at  DATETIME            NOT NULL,
    modified_at DATETIME            NOT NULL,
    walked      BOOLEAN             NOT NULL DEFAULT 1
);
