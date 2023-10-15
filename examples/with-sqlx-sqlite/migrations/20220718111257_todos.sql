CREATE TABLE IF NOT EXISTS todos
(
    uuid        TEXT PRIMARY KEY    NOT NULL,
    task        TEXT                NOT NULL,
    done        BOOLEAN             NOT NULL DEFAULT 0
);