CREATE TABLE IF NOT EXISTS cards
(
    id                  SERIAL PRIMARY KEY         NOT NULL,
    filename            VARCHAR(1000)               NOT NULL,
    name_en             VARCHAR(1000)               NOT NULL,
    description_en      TEXT                        NOT NULL,
    name_ru             VARCHAR(1000)               NOT NULL,
    description_ru      TEXT                        NOT NULL,
    telegram_file_id_en VARCHAR(1000),
    telegram_file_id_ru VARCHAR(1000)             
);
