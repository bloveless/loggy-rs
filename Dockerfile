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

USER loggy

RUN mkdir test-dir && chown -R loggy:loggy ./test-dir

CMD "/home/loggy/loggy-rs /home/loggy/test-dir"
