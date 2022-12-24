pub struct Transaction {
    pub from_address: PublicKey,
    pub to_address: PublicKey,
    pub amount: u64,
    pub signature: Signature
}

pub struct BlockChain {
    pub transactions: Vec<Transaction>,
}

pub struct Signature(pub String);

pub struct PublicKey(pub String);
