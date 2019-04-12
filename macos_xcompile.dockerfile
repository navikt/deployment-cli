FROM rust

WORKDIR /
RUN apt update
RUN apt install -y wget git clang
RUN rustup target add x86_64-apple-darwin
RUN git clone https://github.com/tpoechtrager/osxcross

WORKDIR /osxcross

ENV PKG_CONFIG_ALLOW_CROSS=1
ENV TARGET_CC="/usr/local/osx-ndk-x86/bin/x86_64-apple-darwin15-cc"
ENV UNATTENDED=yes
ENV OSX_VERSION_MIN=10.7

RUN wget https://s3.dockerproject.org/darwin/v2/MacOSX10.11.sdk.tar.xz
RUN mv MacOSX10.11.sdk.tar.xz tarballs/
RUN ./build.sh
RUN mkdir -p /usr/local/osx-ndk-x86
RUN mv target/* /usr/local/osx-ndk-x86
COPY .circleci/cargo_config .cargo/config

RUN USER=root cargo new --bin deployment-cli
WORKDIR deployment-cli
COPY Cargo.toml .
#RUN cargo build --release --target x86_64-apple-darwin

COPY src src

RUN cargo build --release --target x86_64-apple-darwin

