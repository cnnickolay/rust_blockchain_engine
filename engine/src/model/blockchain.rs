use std::fmt::{Display, Formatter};

#[derive(Clone, PartialEq)]
pub struct Transaction {
    pub from_address: PublicKeyStr,
    pub to_address: PublicKeyStr,
    pub amount: u64,
}

pub struct BlockChain {
    pub initial_block: InitialBlock,
    pub blocks: Vec<Block>,
}

pub struct InitialBlock {
    pub address: PublicKeyStr,
    pub balance: u64,
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

#[derive(Clone, PartialEq)]
pub struct PublicKeyStr(pub HexString);

#[derive(Clone, PartialEq)]
pub struct HexString(pub String);

impl Display for PublicKeyStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.0)
    }
}