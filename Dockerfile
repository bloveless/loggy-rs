###########################################################
##### Builder
###########################################################

FROM rust:slim-buster AS builder

RUN mkdir /app

WORKDIR /app

COPY . /app

RUN cargo build --release

###########################################################
##### Release
###########################################################

FROM debian:buster-slim

RUN useradd --create-home loggy

WORKDIR /home/loggy

COPY --from=builder --chown=loggy:loggy /app/target/release/loggy-rs /home/loggy/loggy-rs

RUN mkdir /data && chown -R loggy:loggy /data

USER loggy

CMD ["/home/loggy/loggy-rs", "/data"]
