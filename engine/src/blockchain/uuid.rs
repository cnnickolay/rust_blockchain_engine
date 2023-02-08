use serde::{Deserialize, Serialize};
use uuid::Uuid as _Uuid;

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize)]
pub struct Uuid(pub String);

impl Uuid {
    pub fn new(uuid: &str) -> Self {
        Uuid(uuid.to_string())
    }

    pub fn generate() -> Self {
        Uuid(_Uuid::new_v4().to_string())
    }
}
