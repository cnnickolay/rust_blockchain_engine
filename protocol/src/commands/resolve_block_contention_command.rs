use serde::{Serialize, Deserialize};

/**
 * This request is sent from one validator to another to show that it admits the right
 * of the receiver validator to add next block to the blockchain
 */
#[derive(Serialize, Deserialize, Debug)]
pub struct Vote {
    pub block_chain_tip: String,

    // Combination of blockchain tip hash and public key of the current validator
    // Signature is important to avoid fake votes being sent
    pub block_chain_tip_signature: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VoteReply {
    pub block: String // cbor of the block that will be added to the blockchain
}