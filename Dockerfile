FROM rust:1.65.0 as dependency

# KAFKA dependency need this lib
RUN apt-get update && apt-get -y install cmake protobuf-compiler 

# CARGO UPDATE/AUDIT need thios lib
RUN apt-get -y install libssl-dev
RUN cargo build

FROM rust:1.65.0 as builder
WORKDIR /app
COPY . .
COPY --from=dependency /app/target /app/target
RUN cargo build --release

FROM rust:1.65.0 as deploy
WORKDIR /app
COPY --from=builder /app/target/release/http /app
RUN chmod +x http
EXPOSE 5002
CMD [ "http" ]