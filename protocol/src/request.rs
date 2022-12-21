use crate::{internal::{InternalRequest}, external::{ExternalRequest}};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    Internal (InternalRequest),
    External (ExternalRequest)
}
