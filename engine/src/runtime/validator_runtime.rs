use protocol::request::{Request, Response};
use crate::blockchain::uuid::Uuid;
use super::{configuration::Configuration, validator_state::ValidatorState};

pub struct ValidatorRuntime {
    pub configuration: Configuration,
    pub state: ValidatorState,
    pub event: Event,
    pub processed_events: Vec<Event>
}

impl ValidatorRuntime {
    pub fn new(configuration: Configuration) -> ValidatorRuntime {
        ValidatorRuntime { 
            configuration, 
            state: ValidatorState::StartUp, 
            event: Event::new(EventType::InternalCycleEvent),
            processed_events: Vec::new()
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
    pub response: Option<Response>
}