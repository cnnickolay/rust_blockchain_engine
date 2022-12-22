
/**
 * A runtime configuration for current node
 */
pub struct Configuration {
    ip: String,
    port: u16,
    node_type: NodeType,
}

impl Configuration {
    pub fn new(ip: &str, port: u16, node_type: NodeType) -> Self {
        Configuration {
            ip: ip.to_string(),
            port,
            node_type,
        }
    }
}

pub enum NodeType {
    /**
     * Coordinator node is responsible for receiving a request from the user and passing it to validators
     */
    Coordinator {
        validators: Vec<ValidatorAddress>
    },
    /**
     * Validator node is responsible for receiving a request from the coordinator node and validating it
     */
    Validator
}

pub struct ValidatorAddress(String);

impl NodeType {
    pub fn new_coordinator() -> Self {
        Self::Coordinator { validators: Vec::new() }
    }
}