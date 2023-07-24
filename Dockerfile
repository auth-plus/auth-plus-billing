FROM rust:1.65.0 as dependency
WORKDIR /app
# dependencies need this lib
RUN apt-get update && apt-get -y install cmake protobuf-compiler libssl-dev pkg-config
COPY . .
RUN cargo build --release
RUN cargo build --release --bin  kafka
RUN cargo build --release --bin  cronjob

FROM ubuntu:22.04 as deploy
WORKDIR /app
COPY --from=dependency /app/target/release/http /app
COPY --from=dependency /app/target/release/kafka /app
RUN apt-get update && apt-get -y install wget
RUN wget http://nz2.archive.ubuntu.com/ubuntu/pool/main/o/openssl/libssl1.1_1.1.1f-1ubuntu2.19_amd64.deb

RUN dpkg -i libssl1.1_1.1.1f-1ubuntu2.19_amd64.deb
EXPOSE 5002
