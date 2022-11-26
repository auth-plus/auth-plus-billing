FROM rust:1.64.0 as dependency
WORKDIR /app
COPY . .
RUN cargo build

FROM alpine:3.16.2 as deploy
WORKDIR /app
COPY --from=builder /app/target/release/http /app/http
RUN ./http
EXPOSE 5002