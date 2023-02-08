use std::{sync::{mpsc, Arc, Mutex}, rc::Rc, thread, ops::Deref, time::Duration};

use super::{
    configuration::{Configuration, ValidatorAddress, ValidatorReference},
    validator_state::ValidatorState,
};
use crate::{
    blockchain::{blockchain::BlockChain, uuid::Uuid},
    model::{PublicKeyStr, requests::{Request, Response, ResponseWithRequests, InternalRequest, CommandRequest, ValidatorStartedRequest}}, client_wrappers::{ClientWrapper, ClientWrapperImpl, send}, orchestrator::RequestProcessor
};
use anyhow::Result;
use futures::executor;
use log::{debug, trace, info};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

pub struct ValidatorRuntime {
    pub configuration: Configuration,
    pub state: ValidatorState,
    pub validators: Vec<ValidatorReference>,
    pub processed_events: Vec<Event>,
    pub blockchain: BlockChain,
}

impl ValidatorRuntime {
    pub fn new(configuration: Configuration, blockchain: BlockChain, validators: Vec<ValidatorReference>) -> ValidatorRuntime {
        ValidatorRuntime {
            configuration,
            state: ValidatorState::StartUp,
            validators,
            processed_events: Vec::new(),
            blockchain,
        }
    }

    pub async fn run(self) -> () {
        info!("Running validator on {}", self.configuration.address());
        let listener = TcpListener::bind(self.configuration.address()).await.unwrap();

        let (socket_request_sender, socket_request_receiver) = mpsc::channel::<(Request, Option<TcpStream>)>();
        let (internal_request_sender, internal_request_receiver) = mpsc::channel::<InternalRequest>();

        // sending ValidatorStarted message to itself
        socket_request_sender.send(
            (
                CommandRequest::ValidatorStarted(ValidatorStartedRequest).to_request(&self.configuration.validator()),
                None
            )
        ).unwrap();

        let validator = Arc::new(Mutex::new(self));

        let socket_reader_future = tokio::spawn(async move {
            loop {
                let (mut socket, _) = listener.accept().await.unwrap();
                trace!("New connection opened");

                let request_sender_from_socket = socket_request_sender.clone();

                tokio::spawn(async move {
                    let request = ValidatorRuntime::receive_and_parse(&mut socket).await.unwrap();
                    info!("Received request: {:?}", request);
                    request_sender_from_socket.send((request, Some(socket))).unwrap();
                });
            }
        });

        let internal_request_validator = validator.clone();
        let internal_sub_request_sender = internal_request_sender.clone();
        let internal_request_future = tokio::spawn(async move {
            loop {

                for InternalRequest {request: internal_request, destination_validator: destination} in &internal_request_receiver {
                    debug!("Internal request received: {}", internal_request.command.name());
    
                    let internal_request_validator = internal_request_validator.clone();
                    let internal_sub_request_sender = internal_sub_request_sender.clone();

                    thread::spawn(move || {
                        debug!("Sending internal request to {}", destination.address.0);
                        let internal_response = send(&destination.address, &internal_request).unwrap();
        
                        let internal_sub_requests = RequestProcessor::prod().process_response(
                            &internal_request, 
                            &internal_response, 
                            &mut internal_request_validator.lock().unwrap()
                        ).unwrap();
    
                        for internal_sub_request in internal_sub_requests {
                            internal_sub_request_sender.send(internal_sub_request).unwrap();
                        }
                    });
                }
            }
        });

        let external_request_validator = validator.clone();
        let external_request_future = tokio::spawn(async move {
            loop {
                socket_request_receiver.iter().for_each(|(request, socket)| {
                    info!("Request added to queue: {}", request.command.name());
        
                    let ResponseWithRequests { response, internal_requests } = 
                            RequestProcessor::prod()
                                .next_request(&request, &mut external_request_validator.lock().unwrap())
                                .expect(&format!("Error happened while processing external request: {:?}", request));
    
                    debug!("Response generated: {:?}", response);
    
                    if let Some(mut socket) = socket {
                        let result = serde_cbor::to_vec(&response).unwrap();
                        thread::spawn(move || {
                            executor::block_on(socket.write(&result)).unwrap();
                            debug!("Response sent out");
                        });
                    };
            
                    debug!("Total internal requests: {}", internal_requests.len());
                    for internal_request in internal_requests {
                        debug!("Internal request to {:?}: {:?}", internal_request.destination_validator, internal_request.request);
                        internal_request_sender.send(internal_request).unwrap();
                    }
                });
            }
        });

        internal_request_future.await.unwrap();
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

    pub fn add_validators(&mut self, new_validators: &[ValidatorReference]) {
        let new_distinct_validators = new_validators.iter().filter(
            |ValidatorReference {
                 pk: validator_pub_key,
                 address: validator_addr,
             }| {
                *validator_pub_key != self.configuration.validator_public_key
                    && self
                        .validators
                        .iter()
                        .find(
                            |ValidatorReference {
                                 pk: existing_validator_pub_key,
                                 address: existing_validator_addr,
                             }| {
                                existing_validator_pub_key == validator_pub_key
                            },
                        )
                        .is_none()
            },
        );
        self.validators
            .extend(Vec::from_iter(new_distinct_validators.cloned()));
    }

    pub fn find_validator_address_by_key(&self, key: &PublicKeyStr) -> Option<ValidatorAddress> {
        self.validators.iter().find_map(
            |ValidatorReference {
                 pk: v_pub_k,
                 address: v_addr,
             }| {
                if v_pub_k == key {
                    Some(v_addr.clone())
                } else {
                    None
                }
            },
        )
    }

    pub fn remove_validator(&mut self, pk: &PublicKeyStr) {
        let index = self
            .validators
            .iter()
            .enumerate()
            .find_map(
                |(idx, validator)| {
                    if validator.pk == *pk {
                        Some(idx)
                    } else {
                        None
                    }
                },
            );
        if let Some(index) = index {
            self.validators.remove(index);
            debug!("Validator {} removed", pk);
        }
    }
}

pub struct Event {
    pub event_id: String,
    pub event: EventType,
    pub outgoing_requests: Vec<Request>,
}

impl Event {
    pub fn new(event: EventType) -> Event {
        let event_id = Uuid::generate().0;
        Event {
            event_id,
            event,
            outgoing_requests: Vec::new(),
        }
    }
}

pub enum EventType {
    InternalCycleEvent,
    RequestReceivedEvent,
}

pub struct RequestEvent {
    pub request: Request,
    pub response: Option<Response>,
}
