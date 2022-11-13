CREATE TABLE IF NOT EXISTS images (
    id          UUID    PRIMARY KEY NOT NULL,
    image       TEXT                NOT NULL, 
    created_at  TIMESTAMPZ          NOT NULL,
    modified_at TIMESTAMPZ          NOT NULL,
    walked      BOOLEAN             NOT NULL DEFAULT 1
);
