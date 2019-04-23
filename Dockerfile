FROM alpine

RUN apk --no-cache add ca-certificates curl git openssh-client openssl curl

COPY deployment-cli /usr/bin/deployment-cli
