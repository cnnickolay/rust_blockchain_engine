# Custom Blockchain and Crypto currency engine
Implementation of a custom cryptocurrency on Rust.

# What's implemented as of 14th January 2023
    * UTxO model is implemented, transactions balancing, signing, submitting.
    * Support for multiple validators.
    * Simple consensus between validators.

There are CLI apps in bins directory which can be used for spinning app a blockchain engine and sending client transactions.

1. Build everything with ```cargo build --all```

2. Run three validator nodes in separate terminals \
Validator 1 `./run-validator-1.sh` \
Validator 2 `./run-validator-2.sh` \
Validator 3 `./run-validator-2.sh`

1. Balance, sign and submit transaction
    ```
    . ./sign_and_submit_transaction.sh
    ```
2. You should see something like 
    ```
    Success { request_id: "d6bc280c-50a4-4ffc-9114-0cfeea54a036", response: CommitTransactionResponse { blockchain_hash: "14b2af35e88161ade5d58b0591569aae2b76c117dfd2b472e861541bb33b728c" } }
    ```

3. Print blockchain in all nodes
Make sure all blocks have 3 validations each and they are identical in all nodes

    4.1. First node `target/debug/client_print_blockchain -d 0.0.0.0:9065`

    4.2. Second node `target/debug/client_print_blockchain -d 0.0.0.0:9067`

    4.3. Third node `target/debug/client_print_blockchain -d 0.0.0.0:9068`

    **Example of client_print_blockchain output**

    ```
    N. Block 1054a462703b08b0311d11d35386d93c0f1b8092cbc3e861ee4cabb1441fd995
    Input UTxOs:
        Input 1:
        Addr: 3082010a0282010100ba....f7a1c148190203010001
        Amount: 89
    Output UTxOs:
        Output 1:
        Addr: 3082010a0282010100ba....f7a1c148190203010001
        Amount: 78
        Output 2:
        Addr: 3082010a0282010100b9....73909d53bb0203010001
        Amount: 11
    Transaction signature: 3e589900d3b254639c74....476bb99cca49b9133cb6
    Confirmations (total 3):
        Confirmation 1:
        Validator Id: 3082010a0282010100a5....d378a976030203010001
        Signature: 5e28af5493f0cfbc7846....7074937795f523e5a06e
        Confirmation 2:
        Validator Id: 3082010a0282010100a7....e38a60c1a70203010001
        Signature: 606bb7b63a3b72b2a212....08c3dd42ba606d8e5033
        Confirmation 3:
        Validator Id: 3082010a0282010100c8....3a2803538b0203010001
        Signature: 4f3078cc45758cbe16c0....f795228243dadda629c1
    ```

1. Try to send the same transaction again (redo only 3rd bullet), and you'll see the following
    ```
    # ensure this variable is defined
    echo $BALANCED_TRANSACTION

    # try to submit already committed transaction
    target/debug/client_commit_transaction --private-key "$(<test-data/root_private_key)" --cbor $BALANCED_TRANSACTION
    ```

    This is the result you should observe
    ```
    External(Error { msg: "Utxo a528b2c8ff24d719973b1a549edc2e0891afa8f923d336f02daa39232c850179 has already been spent" })
    ```

# Questions and considerations
Should you have any questions in regards with this project, you can reach out for me at `nickolayc@gmail.com`
