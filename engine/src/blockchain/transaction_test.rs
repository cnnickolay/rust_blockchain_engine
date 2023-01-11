#[cfg(test)]
mod tests {
    use crate::{blockchain::{transaction::Transaction, blockchain::BlockChain, utxo::UnspentOutput}, encryption::{generate_rsa_keypair_custom}};

    #[test]
    fn balance_transaction_happy_path() {
        let (validator_private_key, validator_public_key) = &generate_rsa_keypair_custom().unwrap();
        let (priv_1, pub_1) = &generate_rsa_keypair_custom().unwrap();
        let (priv_2, pub_2) = &generate_rsa_keypair_custom().unwrap();
        
        let mut blockchain = BlockChain::new(validator_public_key, UnspentOutput::new(&pub_1, 10));
        assert_eq!(blockchain.blocks.len(), 0, "Number of transactions is wrong");
        
        // first transaction
        let transaction = Transaction::new(&pub_1, &pub_2, 10)
            .balance_transaction(&blockchain).unwrap();

        assert_eq!(transaction.inputs.len(), 1, "Number of inputs is wrong");
        assert_eq!(transaction.inputs[0].address, *pub_1, "From address is wrong");
        assert_eq!(transaction.inputs[0].amount, 10, "Input amount is wrong");
        assert_eq!(transaction.outputs.len(), 1, "Number of outputs is wrong");
        assert_eq!(transaction.outputs[0].address, *pub_2, "To address is wrong");
        assert_eq!(transaction.outputs[0].amount, 10, "Output amount is wrong");

        let signed_transaction = transaction.sign(&priv_1.try_into().unwrap()).unwrap();

        let block = blockchain.commit_transaction(&signed_transaction, validator_private_key).unwrap();
        assert_eq!(blockchain.blocks.len(), 1, "Number of transactions is wrong");
        assert_eq!(block.validator_signatures.len(), 1, "Number of signatures is wrong");
        assert_eq!(block.validator_signatures[0].validator_public_key, *validator_public_key, "Number of signatures is wrong");

        // second transaction, with change
        let transaction = Transaction::new(&pub_2, &pub_1, 5)
            .balance_transaction(&blockchain)
            .unwrap()
            .sign(&priv_2.try_into().unwrap())
            .unwrap()
            .commit(&mut blockchain, validator_private_key)
            .unwrap()
            .transaction;

        assert_eq!(blockchain.blocks.len(), 2, "Number of transactions is wrong");
        
        assert_eq!(transaction.inputs().len(), 1, "Number of inputs is wrong");
        assert_eq!(transaction.inputs()[0].address, *pub_2, "From address is wrong");
        assert_eq!(transaction.inputs()[0].amount, 10, "Input amount is wrong");
        assert_eq!(transaction.outputs().len(), 2, "Number of outputs is wrong");
        assert_eq!(transaction.outputs()[0].address, *pub_2, "Receiver address is wrong");
        assert_eq!(transaction.outputs()[0].amount, 5, "Output amount is wrong");
        assert_eq!(transaction.outputs()[1].address, *pub_1, "Change address is wrong");
        assert_eq!(transaction.outputs()[1].amount, 5, "Output amount is wrong");

        // third transaction
        let transaction = Transaction::new(&pub_2, &pub_1, 5)
            .balance_transaction(&blockchain)
            .unwrap()
            .sign(&priv_2.try_into().unwrap())
            .unwrap()
            .commit(&mut blockchain, validator_private_key)
            .unwrap();
        assert_eq!(blockchain.blocks.len(), 3, "Number of transactions is wrong");

        // fourth transaction
        let transaction = Transaction::new(&pub_1, &pub_2, 8)
            .balance_transaction(&blockchain)
            .unwrap()
            .sign(&priv_1.try_into().unwrap())
            .unwrap()
            .commit(&mut blockchain, validator_private_key)
            .unwrap()
            .transaction;

        assert_eq!(blockchain.blocks.len(), 4, "Number of transactions is wrong");

        assert_eq!(transaction.inputs().len(), 2, "Number of inputs is wrong");
        assert_eq!(transaction.inputs()[0].address, *pub_1, "From address is wrong");
        assert_eq!(transaction.inputs()[0].amount, 5, "First input amount is wrong");
        assert_eq!(transaction.inputs()[1].address, *pub_1, "From address is wrong");
        assert_eq!(transaction.inputs()[1].amount, 5, "Second input amount is wrong");
        assert_eq!(transaction.outputs().len(), 2, "Number of outputs is wrong");
        assert_eq!(transaction.outputs()[0].address, *pub_1, "Receiver address is wrong");
        assert_eq!(transaction.outputs()[0].amount, 2, "Output amount is wrong");
        assert_eq!(transaction.outputs()[1].address, *pub_2, "Change address is wrong");
        assert_eq!(transaction.outputs()[1].amount, 8, "Change amount is wrong");
    }
    
    #[test]
    fn balance_transaction_not_enough_funds() {
    }

}