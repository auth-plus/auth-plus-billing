FROM rust:1.64.0 as dependency

# KAFKA dependency need this lib
RUN apt-get update && apt-get -y install cmake protobuf-compiler

WORKDIR /app
COPY . .
RUN cargo build

FROM alpine:3.16.2 as deploy
WORKDIR /app
COPY --from=builder /app/target/release/http /app/http
RUN ./http
EXPOSE 5002