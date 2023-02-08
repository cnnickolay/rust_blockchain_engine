use std::{env, rc::Rc, sync::Arc};

use engine::{encryption::generate_rsa_keypair_custom, blockchain::utxo::UnspentOutput, runtime::{configuration::Configuration, validator_runtime::ValidatorRuntime}, client_wrappers::{ClientWrapperImpl, ClientWrapper}, orchestrator::RequestProcessor};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "trace")
    }
    env_logger::init();

    let (_, initial_pk) = generate_rsa_keypair_custom()?;

    let validator_1 = Configuration::generate_new("0.0.0.0", 8055)?;
    let validator_1_reference = validator_1.validator_ref();
    let validator_2 = Configuration::generate_new("0.0.0.0", 8056)?;

    let initial_pk_cloned = initial_pk.clone();
    let validator_1_future = tokio::spawn(async move {
        Configuration::generate_new("0.0.0.0", 8055).unwrap()
            .to_runtime(UnspentOutput::new(&initial_pk_cloned, 100).to_blockchain(), Vec::new())
            .run()
            .await
    });

    let initial_pk_cloned = initial_pk.clone();
    let validator_2_future = tokio::spawn(async move {
        validator_2
            .to_runtime(UnspentOutput::new(&initial_pk_cloned, 100).to_blockchain(), vec![validator_1_reference])
            .run()
            .await;
    });

    println!("Server stopped");

    let (validator_1_future_result, validator_2_future_result) = tokio::join!(validator_1_future, validator_2_future);
    validator_1_future_result?;
    validator_2_future_result?;

    Ok(())
}