use serde::{Serialize, Deserialize};


/**
 * This request is meant for resolving which of the two blocks will be the new tip of the blockchain.
 */
#[derive(Serialize, Deserialize, Debug)]
pub struct ResolveBlockContention {
    
    pub block: String
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ResolveBlockContentionResult {
    /**
     * This result means that the sending validator has won the competition for adding the block to the blockchain
     */
    ThisSideWins {
        remote_block: String
    },

    /**
     * This result means that another validator has won the competition for adding the block to the blockchain
     */
    OtherSideWins {
        remote_block: String
    }
}