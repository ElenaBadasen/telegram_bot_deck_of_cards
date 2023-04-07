CREATE TABLE IF NOT EXISTS bots_subscribers
(
    id              SERIAL PRIMARY KEY         NOT NULL,
    bot_id          INTEGER                    NOT NULL, 
    subscriber_id   INTEGER                    NOT NULL,
    CONSTRAINT fk_bot
      FOREIGN KEY(bot_id) 
	    REFERENCES bots(id),
    CONSTRAINT fk_subscriber
      FOREIGN KEY(subscriber_id) 
	    REFERENCES subscribers(id)
);

CREATE UNIQUE INDEX index_bots_subscribers_on_bot_id_and_subscriber_id 
ON bots_subscribers(bot_id, subscriber_id);