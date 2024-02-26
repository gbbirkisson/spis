CREATE TABLE IF NOT EXISTS media (
    id          UUID    PRIMARY KEY NOT NULL,
    path        TEXT                NOT NULL, 
    taken_at    DATETIME            NOT NULL,
    walked      BOOLEAN             NOT NULL DEFAULT 1
);
