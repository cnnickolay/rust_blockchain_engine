use protocol::external::{ExternalRequest, UserCommand};

use super::model::{State, Block};

impl State {
    pub fn new() -> State {
        State {
            block_chain: Vec::new()
        }
    }

    pub fn add_block(&mut self, request: ExternalRequest) {
        self.block_chain.push(Block::new(request.command))
    }
}

impl Block {
    pub fn new(user_command: UserCommand) -> Self {
        Block {
            user_command,
        }
    }
}