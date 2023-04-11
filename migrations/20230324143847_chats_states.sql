CREATE TYPE language AS ENUM ('en', 'ru');
CREATE TABLE IF NOT EXISTS chats_states
(
    id                    SERIAL PRIMARY KEY          NOT NULL,
    bot_id                INTEGER                     NOT NULL,
    subscriber_id         INTEGER                     NOT NULL,
    drawn_cards           INTEGER []                  NOT NULL,
    language              language                    NOT NULL DEFAULT 'en',
    descriptions_format   INTEGER                     NOT NULL DEFAULT 0,
    CONSTRAINT fk_bot
      FOREIGN KEY(bot_id) 
	    REFERENCES bots(id),
    CONSTRAINT fk_subscriber
      FOREIGN KEY(subscriber_id) 
	    REFERENCES subscribers(id)
);

CREATE UNIQUE INDEX index_chats_states_on_bot_id_and_subscriber_id 
ON chats_states(bot_id, subscriber_id);
CREATE UNIQUE INDEX index_chats_states_on_subscriber_id 
ON chats_states(subscriber_id);
CREATE INDEX index_chats_states_on_bot_id
ON chats_states(bot_id);