FROM rust:1.68 as build
WORKDIR /usr/src/app

COPY src/ ./src/
COPY Cargo.toml ./
COPY Cargo.lock ./

RUN cargo build --release

FROM debian:buster

RUN apt-get update && \
    apt-get install -y openssl && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

COPY --from=build /usr/src/app/target/release/ /usr/local/bin/

CMD /usr/local/bin/iotawatt-exporter