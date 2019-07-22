FROM alpine

RUN apk --no-cache add ca-certificates curl git openssh-client openssl curl docker bash jq

ENV RUST_BACKTRACE=1

COPY deployment-cli /usr/bin/deployment-cli
