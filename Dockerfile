# syntax = docker/dockerfile:1.2

FROM clux/muslrust:stable as build

COPY . /volume
RUN --mount=type=cache,target=/root/.cargo/registry --mount=type=cache,target=/volume/target \
    cargo b --release --target x86_64-unknown-linux-musl && \
    cp target/x86_64-unknown-linux-musl/release/tg-media-bot tg-media-bot

FROM gcr.io/distroless/static

COPY --from=build /volume/tg-media-bot /tg-media-bot
COPY telegram_token ./telegram_token

#RUN --mount=type=secret,id=telegram_token export "TELOXIDE_TOKEN=${cat /run/secrets/telegram_token}"

CMD ["/tg-media-bot"]
