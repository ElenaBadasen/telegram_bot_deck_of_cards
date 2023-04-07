CREATE TABLE IF NOT EXISTS bots
(
    id          SERIAL PRIMARY KEY  NOT NULL,
    name        VARCHAR(250)        NOT NULL,
    active      BOOLEAN             NOT NULL DEFAULT TRUE,
    key         VARCHAR(250)        NOT NULL
);