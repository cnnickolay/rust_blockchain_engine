use crate::{
    blockchain::{blockchain::BlockChain, utxo::UnspentOutput},
    client::send_bytes,
    model::{PrivateKeyStr, PublicKeyStr},
    request_handlers::handle_request,
    response_handlers::handle_response,
    runtime::configuration::{Configuration, ValidatorAddress, ValidatorReference},
};
use anyhow::Result;
use futures::{
    channel::oneshot::{self, Sender},
    future::lazy,
    FutureExt,
};
use log::{debug, error, info, trace};
use protocol::{
    request::{CommandRequest, Request},
    request::{CommandResponse, Response, ResponseBody},
};
use rsa::{RsaPrivateKey, RsaPublicKey};
use std::{
    collections::HashSet,
    sync::{mpsc, Arc, Mutex},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

pub async fn run_node(
    host: String,
    port: u16,
    remote_validator_opt: Option<&str>,
    private_key: &str,
    public_key: &str,
) -> Result<()> {
    let listener = TcpListener::bind(format!("{}:{}", host, port)).await?;

    info!("Validator node is running on {}:{}", host, port);

    // a hardcoded public key for the initial block
    let pub_key_str = PublicKeyStr::from_str("3082010a0282010100bae507cd770270df4e249dfde2a89fe9f6abccbb2e56a82f7ce370c763355c09b596d14734d9e225c3ee913f442aa75ea3dba35edb0ae20bdac52ab8f5451c2dafb93a59dccef395f2dce4069880d8ac1f25300edd09fe61cfe0734efb789fc0c8d8d9f1f916165713f394fc275c2652c69fdbddd43e14b12971683e918dcfb0b97511cb36132acb156235d93aac5f3b46b7ae10445c757ed3ebc6c81c9ae8d496e2ecf948c70a100a10badc68558d121a1240df756c55c8c4c90990c826646dec4e319b55ce15c1e24d9273ea560aeb09834caa0827f99668e81d865a12e059ddaf5987601a7d6c5bfaf14e72182eb83369883a01f9eeb4b09261f7a1c148190203010001");
    let pub_key = RsaPublicKey::try_from(&pub_key_str)?;

    let validator_private_key = PrivateKeyStr::from_str(private_key);
    let validator_public_key = PublicKeyStr::from_str(public_key);
    RsaPublicKey::try_from(&validator_public_key).expect("Public key provided is wrong");
    RsaPrivateKey::try_from(&validator_private_key).expect("Private key provided is wrong");

    let blockchain = BlockChain::new(UnspentOutput::initial_utxo(&pub_key_str, 100));
    let rt = Configuration::new(&host, port, &validator_private_key).to_runtime(blockchain, vec![]);
    let validator = rt.configuration.validator();
    let rt = Arc::new(Mutex::new(rt));

    let processed_requests = Arc::new(Mutex::new(HashSet::<String>::new()));

    let (socket_sender, socket_receiver) = mpsc::channel::<(Request, Sender<Response>)>();
    let (requests_sender, requests_receiver) = mpsc::channel::<(ValidatorReference, Request)>();

    // Register current validator with other validators
    if let Some(remote_validator) = remote_validator_opt {
        info!("Connecting to remote validator {}", remote_validator);
        let request = CommandRequest::new_on_board_command(
            &format!("{}:{}", host, port),
            &validator_public_key.0 .0,
        )
        .to_request(&validator);

        requests_sender
            .send((
                ValidatorReference {
                    pk: PublicKeyStr::from_str("not-necessary-here"),
                    address: ValidatorAddress(remote_validator.to_owned()),
                },
                request,
            ))
            .unwrap();
    };

    // Handling triggered requests
    let rt_1 = rt.clone();
    let requests_sender_1 = requests_sender.clone();
    tokio::spawn(async move {
        loop {
            let requests_sender = requests_sender_1.clone();
            requests_receiver.iter().for_each(|(val_ref, request)| {
                let request_id = request.request_id.clone();
                debug!("Sending triggered request with id {}", request_id);

                let mut rt = rt_1.lock().unwrap();

                match send_bytes(&val_ref.address.0, &request) {
                    Ok(response) => {
                        let requests = handle_response(&mut rt, &request_id, &response)
                        .unwrap_or_else(|err| {
                            error !("{}", err); 
                            Vec::new()
                        });
                        for r in requests {
                            requests_sender.send(r).unwrap();
                        }
                    },
                    Err(err) => {
                        error!("Unable to reach validator by address {} because of: {}. Validator will be removed", val_ref.address.0, err);
                        // need better solution, maybe remove after several failed attempts to send request
                        rt.remove_validator(&val_ref.pk);
                    },
                }
            });
        }
    });

    // reading incoming messages from the socket
    let validator_1 = validator.clone();
    tokio::spawn(async move {
        loop {
            let (mut socket, _) = listener.accept().await.unwrap();
            trace!("New connection opened");

            let tx = socket_sender.clone();

            let processed_requests = processed_requests.clone();
            let validator = validator_1.clone();
            tokio::spawn(async move {
                let request = receive_and_parse(&mut socket).await.unwrap();

                let response = if processed_requests
                    .lock()
                    .unwrap()
                    .contains(&request.request_id)
                {
                    Response {
                        orig_request_id: request.request_id.to_owned(),
                        replier: validator,
                        body: ResponseBody::Success(CommandResponse::Nothing),
                    }
                } else {
                    processed_requests
                        .lock()
                        .unwrap()
                        .insert(request.request_id.to_owned());
                    debug!("Received request: {:?}", request);

                    let (callback_sender, callback_receiver) = oneshot::channel::<Response>();
                    tx.send((request, callback_sender)).unwrap();
                    callback_receiver
                        .map(|response| {
                            trace!("Response {:?}", response);
                            response
                        })
                        .await
                        .unwrap()
                };

                let result = serde_cbor::to_vec(&response).unwrap();
                socket.write(&result).await.unwrap();
            });
        }
    });

    // handling requests
    let rt_2 = rt.clone();
    let validator_2 = validator.clone();
    lazy(|_| {
        socket_receiver.iter().for_each(|(request, callback)| {
            let (response, sub_requests) = handle_request(&request, &mut rt.lock().unwrap())
                .unwrap_or_else(|e| {
                    // error!("{}", e);
                    let response = Response {
                        orig_request_id: request.request_id.to_owned(),
                        replier: validator_2.clone(),
                        body: ResponseBody::Error {
                            msg: format!("{:?}", e),
                        },
                    };

                    (response, Vec::new())
                });

            callback.send(response).unwrap();

            for req in sub_requests {
                requests_sender.send(req).unwrap();
            }
        });
    })
    .await;

    Ok(())
}

/**
 * Reads request from socket and parses it
 */
pub async fn receive_and_parse(stream: &mut TcpStream) -> Result<Request> {
    let mut len: [u8; 8] = [0; 8];
    let mut buf: [u8; 10240] = [0; 10240];
    stream.read(&mut len).await?;

    let size_received = usize::from_be_bytes(len);
    stream.read(&mut buf).await?;

    let received_msg: Request = serde_cbor::from_slice(&buf[0..size_received])?;
    Ok(received_msg)
}
