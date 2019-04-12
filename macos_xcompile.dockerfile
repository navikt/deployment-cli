FROM rust

WORKDIR /
RUN apt update
RUN apt install -y wget git clang
RUN rustup target add x86_64-apple-darwin
RUN git clone https://github.com/tpoechtrager/osxcross

WORKDIR /osxcross

COPY buildscripts/setup_osxcross.sh .
RUN ./setup_osxcross.sh
COPY buildscripts/cargo_config .cargo/config

RUN USER=root cargo new --bin deployment-cli
WORKDIR deployment-cli

ENV PKG_CONFIG_ALLOW_CROSS=1
ENV TARGET_CC="/usr/local/osx-ndk-x86/bin/x86_64-apple-darwin15-cc"

COPY Cargo.toml .
#RUN cargo build --release --target x86_64-apple-darwin

COPY src src

RUN cargo build --release --target x86_64-apple-darwin

