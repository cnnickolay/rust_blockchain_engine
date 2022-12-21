use crate::{internal::{InternalResponse}, external::{ExternalResponse}};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    Internal (InternalResponse),
    External (ExternalResponse)
}
