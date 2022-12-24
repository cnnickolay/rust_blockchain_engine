

use super::model::{BlockChain};

impl BlockChain {
    pub fn new() -> Self {
        BlockChain {
            transactions: Vec::new()
        }
    }
}
