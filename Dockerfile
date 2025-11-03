FROM rust:alpine

RUN apk add --no-cache libressl-dev musl-dev
WORKDIR /build/
COPY ./Cargo.lock ./Cargo.toml ./
COPY ./src/ ./src/
RUN cargo build --locked --release --bins

FROM alpine

COPY --from=0 /build/target/release/am-rate-bot /main

USER nobody:nogroup
STOPSIGNAL SIGINT

CMD ["/main"]
