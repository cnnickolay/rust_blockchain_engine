# Blockchain engine
Implementation of a custom cryptocurrency on Rust.

# What's implemented as of 2 January 2023
UTxO model is implemented, transactions balancing, signing, submitting.
There are CLI apps in bins directory which can be used for spinning app a blockchain engine and sending client transactions.

0. Build everything with ```cargo build --all```

1. Run three validator nodes in separate terminals
Validator 1 `./run-validator-1.sh`
Validator 2 `./run-validator-2.sh`
Validator 3 `./run-validator-2.sh`

2. Balance, sign and submit transaction
```
./sign_and_submit_transaction.sh
```
3. You should see something like 
```
Success { request_id: "d6bc280c-50a4-4ffc-9114-0cfeea54a036", response: CommitTransactionResponse { blockchain_hash: "14b2af35e88161ade5d58b0591569aae2b76c117dfd2b472e861541bb33b728c" } }
```

4. Print blockchain in all nodes
Make sure all blocks have 3 validations each and they are identical in all nodes
4.1. First node
```
target/debug/client_print_blockchain -d 0.0.0.0:9065
```

4.2. Second node
```
target/debug/client_print_blockchain -d 0.0.0.0:9067
```

4.3. Third node
```
target/debug/client_print_blockchain -d 0.0.0.0:9068
```

5. Try to send the same transaction again (redo only 3rd bullet), and you'll see the following
```
External(Error { msg: "Utxo a528b2c8ff24d719973b1a549edc2e0891afa8f923d336f02daa39232c850179 has already been spent" })
```

# What's to be implemented
Consensus algorithm. It's not gonna be PoS or PoW for now, just a naive and yet working implementation of consensus algorithm.
