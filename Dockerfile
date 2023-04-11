FROM rust:1.68.2-alpine3.17 as build

COPY ./migrations ./migrations
COPY ./pictures ./pictures
COPY ./src ./src
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./sqlx-data.json ./sqlx-data.json

RUN apk update && apk add \
    g++

RUN cargo build --release

FROM alpine:3.17

COPY --from=build /target/release/telegram_bot_deck_of_cards .
COPY ./pictures ./pictures

CMD ["/telegram_bot_deck_of_cards", "/secrets"]