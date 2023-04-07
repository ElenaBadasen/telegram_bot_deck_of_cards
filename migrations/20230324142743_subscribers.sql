CREATE TABLE IF NOT EXISTS subscribers
(
    id          SERIAL PRIMARY KEY          NOT NULL,
    chat_id     VARCHAR(10000)              NOT NULL,
    created_at  TIMESTAMP WITH TIME ZONE    NOT NULL
);

CREATE UNIQUE INDEX index_subscribers_chat_id 
ON subscribers(chat_id);