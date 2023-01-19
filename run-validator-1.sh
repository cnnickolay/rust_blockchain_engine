RUST_LOG=debug \
RUST_BACKTRACE=1 \
target/debug/node --private-key "$(<test-data/validator-1_private_key)" --public-key "$(<test-data/validator-1_public_key)"
