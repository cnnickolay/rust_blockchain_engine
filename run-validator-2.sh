RUST_BACKTRACE=1 target/debug/node --port 9068 --remote-validator 0.0.0.0:9065 --private-key "$(<test-data/validator-2_private_key)" --public-key "$(<test-data/validator-2_public_key)"
