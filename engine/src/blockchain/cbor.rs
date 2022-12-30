pub struct Cbor(pub String);

impl Cbor {
    pub fn new(cbor: &str) -> Self {
        Cbor(cbor.to_owned())
    }
}