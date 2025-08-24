FROM alpine:latest AS builder

ARG DATABASE_URL="postgres://kino:kino@localhost/kino"

RUN apk add --no-cache cargo pkgconf openssl openssl-dev

RUN mkdir -p /app
WORKDIR /app

COPY src src
COPY Cargo.toml .

ENV DATABASE_URL=${DATABASE_URL}
RUN cargo build --release

FROM alpine:latest

RUN apk add --no-cache cargo openssl

COPY --from=builder /app/target/release/kino-api kino-api
COPY .env .

CMD ./kino-api
