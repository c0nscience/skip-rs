FROM node:22-alpine3.21 AS webbuilder

WORKDIR /app

COPY . .

RUN npm ci && npm run prd

FROM rust:alpine3.21 AS builder

RUN apk add -U --no-cache ca-certificates
RUN apk add --no-cache musl-dev openssl-libs-static pkgconfig openssl-dev

WORKDIR /app

COPY . .

RUN cargo build --release

FROM scratch

WORKDIR /app

COPY --from=webbuilder /app/public /app/public
COPY --from=webbuilder /app/templates /app/templates
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /app/target/release/skip-rs /usr/bin/

ENTRYPOINT ["skip-rs"]
