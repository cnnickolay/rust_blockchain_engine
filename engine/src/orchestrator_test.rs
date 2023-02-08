// #[cfg(test)]
// mod test {
//     use anyhow::Result;

//     use crate::{
//         blockchain::{blockchain::BlockChain, utxo::UnspentOutput},
//         client_wrappers::MockClientWrapper,
//         encryption::generate_rsa_keypair_custom,
//         orchestrator::RequestProcessor,
//         runtime::{configuration::{Configuration, ValidatorAddress, ValidatorReference}},
//     };

//     #[test]
//     fn main() -> Result<()> {
//         let mut client_wrapper = MockClientWrapper::new();
//         client_wrapper
//             .expect_send_blockchain_tip_request()
//             .returning(|_, _| Err(anyhow::anyhow!("HAHA, Mock")));

//         let (_, initial_pk) = generate_rsa_keypair_custom()?;
//         let (validator1_sk, validator1_pk) = generate_rsa_keypair_custom()?;

//         let processor = RequestProcessor {
//             client: Box::new(client_wrapper),
//         };

//         let blockchain = BlockChain::new(UnspentOutput::initial_utxo(&initial_pk, 100));

//         let mut rt = Configuration::generate_new("0.0.0.0", 8080)?.to_runtime(blockchain, vec![]);
//         rt.add_validators(&[ValidatorReference {
//             pk: validator1_pk,
//             address: ValidatorAddress("".to_owned()),
//         }]);

//         processor.next_request(&mut rt)
//     }

//     #[test]
//     fn e2e_test() -> Result<()> {
//         let (_, initial_pk) = generate_rsa_keypair_custom()?;

//         let blockchain = UnspentOutput::new(&initial_pk, 100).to_blockchain();
//         let validator = Configuration::generate_new("0.0.0.0", 8081)?.to_runtime(blockchain);
        
//         validator.run()
//     }
// }
