FROM rust:alpine3.16 as builder
WORKDIR /app
COPY . .
# RUN cargo check
RUN cargo build --release

FROM alpine:3.16.2 as deploy
WORKDIR /app
COPY --from=builder /app/target/release/http /app/http
RUN ./http