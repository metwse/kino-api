FROM alpine:latest

RUN apk add --no-cache cargo pkgconf openssl openssl-dev tar wget

RUN mkdir -p /app
WORKDIR /app

RUN wget https://wordnetcode.princeton.edu/3.0/WNdb-3.0.tar.gz
RUN tar -xvzf WNdb-3.0.tar.gz
RUN mv dict wn
RUN rm WNdb-3.0.tar.gz

COPY src src
COPY Cargo.toml .
COPY .env .

CMD cargo run --release
