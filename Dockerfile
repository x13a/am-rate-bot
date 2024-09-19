FROM rust:alpine

RUN apk add --no-cache libressl-dev musl-dev
WORKDIR /build/
COPY ./Cargo.lock ./Cargo.toml ./
COPY ./src/ ./src/
RUN cargo build --locked --release --bins

FROM alpine

COPY --from=0 /build/target/release/am-rate-bot /

USER nobody:nogroup
STOPSIGNAL SIGINT

#ENV HEALTHCHECK_ENABLED 1
#HEALTHCHECK CMD healthy http://127.0.0.1:8000/ping || exit 1

CMD ["/am-rate-bot"]
