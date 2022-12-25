#[derive(Clone)]
pub struct Transaction {
    pub from_address: PublicKeyStr,
    pub to_address: PublicKeyStr,
    pub amount: u64,
}

pub struct BlockChain {
    pub blocks: Vec<Block>,
}

pub struct Block {
    pub prev_hash: String,
    /**
     * Hash combines hashes of both transaction and transaction_signature
     */
    pub hash: String,
    pub transaction: Transaction,
    pub transaction_signature: Signature
}

#[derive(Clone)]
pub struct Signature(pub HexString);

#[derive(Clone)]
pub struct PublicKeyStr(pub HexString);

#[derive(Clone)]
pub struct HexString(pub String);
