###########################################################
##### Builder
###########################################################

FROM rust:1.49.0-slim-buster AS builder

RUN mkdir /app

WORKDIR /app

COPY . /app

RUN cargo build --release

###########################################################
##### Release
###########################################################

FROM debian:buster-slim

WORKDIR /root

COPY --from=builder /app/target/release/loggy-rs /root/loggy-rs

RUN mkdir /data

CMD ["/root/loggy-rs", "/data"]
