FROM rust:1.66 as builder
WORKDIR /usr/src/rust-blockchain
COPY . .
RUN cargo build --release --all

FROM rust:1.66
RUN apt-get update && apt-get install -y jq && rm -rf /var/lib/apt/lists/*
ADD test-data /test-data
ADD /docker-bins /
RUN chmod +x *.sh
COPY --from=builder /usr/src/rust-blockchain/target/release/* /usr/local/bin/

CMD ["/validator.sh"]