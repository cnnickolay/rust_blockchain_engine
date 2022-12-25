pub struct Transaction {
    pub from_address: PublicKeyStr,
    pub to_address: PublicKeyStr,
    pub amount: u64,
    pub signature: Signature
}

pub struct BlockChain {
    pub transactions: Vec<Transaction>,
}

pub struct Signature(pub HexString);

pub struct PublicKeyStr(pub HexString);

pub struct HexString(pub String);
