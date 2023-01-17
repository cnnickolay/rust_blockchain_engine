#!/bin/bash

export BALANCED_TRANSACTION=$(client_balance_transaction -d $VALIDATOR_CALLBACK:9065 --from-address "$(<test-data/root_public_key)" --to-address "$(<test-data/wallet_2_public_key)" -a 11 | jq -r .body.Success.BalanceTransactionResponse.cbor)

client_commit_transaction -d $VALIDATOR_CALLBACK:9065 --private-key "$(<test-data/root_private_key)" --cbor $BALANCED_TRANSACTION

