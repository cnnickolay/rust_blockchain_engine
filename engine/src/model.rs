use protocol::external::UserCommand;


/**
 * Block represents a single element in the blockchain
 */
#[derive(Debug)]
pub struct Block {
    user_command: UserCommand
}

impl Block {
    pub fn new(user_command: UserCommand) -> Self {
        Block {
            user_command,
        }
    }
}