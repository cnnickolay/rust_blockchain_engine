use serde::{Serialize, Deserialize};

use super::uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionId(pub Uuid);

impl TransactionId {
    pub fn new(id: &str) -> TransactionId {
        TransactionId(Uuid::new(id))
    }

    pub fn generate() -> TransactionId {
        TransactionId(Uuid::generate())
    }

}