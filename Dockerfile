FROM rust:1.66 as builder
WORKDIR /usr/src/rust-blockchain
COPY . .
RUN cargo build --release --all

FROM rust:1.66
RUN apt-get update && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/rust-blockchain/target/release/node /usr/local/bin/node
CMD ["node"]
