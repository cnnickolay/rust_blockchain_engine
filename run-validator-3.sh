RUST_LOG=debug \
RUST_BACKTRACE=1 \
target/debug/node --port 9067 --remote-validator 0.0.0.0:9068 --private-key "$(<test-data/validator-3_private_key)" --public-key "$(<test-data/validator-3_public_key)" 