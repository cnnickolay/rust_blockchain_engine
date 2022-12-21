use protocol::external::UserCommand;


/**
 * Block represents a single element in the blockchain
 */
#[derive(Debug)]
pub struct Block {
    pub user_command: UserCommand
}

/**
 * Holds an internal state of the current node
 */
#[derive(Debug)]
 pub struct State {
    pub block_chain: Vec<Block>
}