use crate::{runtime::{validator_state::ValidatorState::{Election, Expanse, StartUp}, validator_runtime::ValidatorRuntime, configuration::{Configuration, ValidatorReference, ValidatorAddress}}, blockchain::{blockchain::BlockChain, utxo::UnspentOutput}, client::Client, client_wrappers::MockClientWrapper, encryption::generate_rsa_keypair_custom};
use anyhow::Result;
use super::client_wrappers::ClientWrapper;

struct RequestProcessor {
    client: Box<dyn ClientWrapper>
}

impl RequestProcessor {
    pub fn next_request(&self, blockchain: &BlockChain, rt: &mut ValidatorRuntime) -> Result<()> {
        match rt.state {
            StartUp => self.synchronize(rt),
            Election => todo!(),
            Expanse => todo!(),
        }
    }
    
    fn synchronize(&self, rt: &ValidatorRuntime) -> Result<()> {
        // 1. find out which blockchain is the dominant on the network (>50% of network should share it)
        let sender = rt.configuration.validator();
        for validator in &rt.configuration.validators {
            let blockchain_tip = self.client.send_blockchain_tip_request(&validator.address, &sender)?;
        }
    
        // 2. synchronize with these nodes
    
        Ok(())
    }
}

#[test]
fn main() -> Result<()> {
    let mut client_wrapper = MockClientWrapper::new();
    client_wrapper.expect_send_blockchain_tip_request().returning(|_, _| Err(anyhow::anyhow!("HAHA, Mock")));

    let (priv_key, pub_key) = generate_rsa_keypair_custom()?;
    let (validator1_sk, validator1_pk) = generate_rsa_keypair_custom()?;
    let processor = RequestProcessor { client: Box::new(client_wrapper) };
    let blockchain = BlockChain::new(UnspentOutput::initial_utxo(&pub_key, 100));
    let mut rt = ValidatorRuntime::new(Configuration::new("0.0.0.0", 8080, &priv_key));
    rt.configuration.add_validators(&[ValidatorReference { pk: validator1_pk, address: ValidatorAddress("".to_owned()) }]);

    processor.next_request(&blockchain, &mut rt)
}