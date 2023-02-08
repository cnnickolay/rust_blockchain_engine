/**
 * Every node broadcasts a Shout message to other validators
 */
pub struct Shout {
    pub block_chain_tip: String,

    // Combination of blockchain tip hash and public key of the current validator
    // Signature is important to avoid fake votes being sent
    pub block_chain_tip_signature: String,
}

/**
 *
 */
pub struct DeclareBlockOwner {}
