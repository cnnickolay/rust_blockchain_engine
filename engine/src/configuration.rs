
/**
 * A runtime configuration for current node
 */
pub struct Configuration {
    pub ip: String,
    pub port: u16,
    pub node_type: NodeType,
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

#[derive(PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct ValidatorAddress(pub String);

impl NodeType {
    pub fn new_coordinator() -> Self {
        Self::Coordinator { validators: Vec::new() }
    }
}