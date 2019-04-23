FROM rust

RUN apt update
RUN apt install -y musl musl-tools openssl
RUN rustup target add x86_64-unknown-linux-musl

RUN USER=root cargo new --bin deployment-cli
WORKDIR deployment-cli
COPY Cargo.toml .
RUN cargo build --release --target x86_64-unknown-linux-musl

COPY src src

RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:3.9
COPY --from=0 /deployment-cli/target/x86_64-unknown-linux-musl/release/deployment-cli /bin/deployment-cli
