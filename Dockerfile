FROM rust:1.65.0 as dependency
WORKDIR /app
# KAFKA dependency need this lib
RUN apt-get update && apt-get -y install cmake protobuf-compiler 
# CARGO UPDATE/AUDIT need thios lib
RUN apt-get -y install libssl-dev
COPY . .
RUN cargo build --release

FROM ubuntu:22.04 as deploy
WORKDIR /app
COPY --from=dependency /app/target/release/http /app
EXPOSE 5002
CMD [ "./http" ]
