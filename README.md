# Blockchain engine
Implementation of a custom cryptocurrency on Rust.

# What's implemented as of 2 January 2023
UTxO model is implemented, transactions balancing, signing, submitting.
There are CLI apps in bins directory which can be used for spinning app a blockchain engine and sending client transactions.

0. Build everything with ```cargo build --all```

1. Run blockchain coordinator (validators are not necessary at this stage)
```
target/debug/node -r "$(<test-data/root_public_key)"
```

2. Balance a transaction
```
target/debug/client_balance_transaction --from-address "$(<test-data/root_public_key)" --to-address "$(<test-data/wallet_2_public_key)" -a 11 | jq -r .External.Success.BalanceTransactionResponse.cbor
```

3. Sign and send balanced transaction
```target/debug/client_commit_transaction --private-key "$(<test-data/root_private_key)" --cbor a3626964782462353165353162362d376661352d343735392d61.....```
This CBOR is too long to keep it in README file, just take it from the previous command.

4. You should see something like 
```
External(Success(CommitTransactionResponse { transaction_id: "8e5fb3cf-3d22-4535-a035-a26f1595ac58" }))
```

5. Try to send the same transaction again (redo only 3rd bullet), and you'll see the following
```
External(Error { msg: "Utxo a528b2c8ff24d719973b1a549edc2e0891afa8f923d336f02daa39232c850179 has already been spent" })
```

# What's to be implemented
Consensus algorithm. It's not gonna be PoS or PoW for now, just a naive and yet working implementation of consensus algorithm.
