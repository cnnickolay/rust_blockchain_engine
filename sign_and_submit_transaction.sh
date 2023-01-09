export BALANCED_TRANSACTION=$(target/debug/client_balance_transaction --from-address "$(<test-data/root_public_key)" --to-address "$(<test-data/wallet_2_public_key)" -a 11 | jq -r .External.Success.BalanceTransactionResponse.cbor)

target/debug/client_commit_transaction --private-key "$(<test-data/root_private_key)" --cbor $BALANCED_TRANSACTION
