export BALANCED_TRANSACTION=$(target/debug/client_balance_transaction -d 0.0.0.0:9068 --from-address "$(<test-data/root_public_key)" --to-address "$(<test-data/wallet_2_public_key)" -a 11 | jq -r .body.Success.BalanceTransactionResponse.cbor)

target/debug/client_commit_transaction -d 0.0.0.0:9068 --private-key "$(<test-data/root_private_key)" --cbor $BALANCED_TRANSACTION
