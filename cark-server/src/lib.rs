pub mod connection;

pub struct Global {
    pub messages: Vec<String>,
    field: cark_common::Field,
}

impl Global {
    pub fn new() -> Self {
        Self {
            messages: vec![],
            field: cark_common::Field::new(20, 20),
        }
    }

    pub fn process(
        &mut self,
        incoming_events: &mut Vec<IncomingEvent>,
        mut outgoing_events: impl FnMut(OutgoingEvent),
    ) {
        for event in incoming_events.drain(..) {
            log::info!("{:?}", &event);
            match event {
                IncomingEvent::Join { connection_id } => {
                    outgoing_events(OutgoingEvent::Joined {
                        connection_id,
                        field: self.field.clone(),
                    });
                }
                IncomingEvent::Message { message } => {
                    self.messages.push(message.clone());
                    outgoing_events(OutgoingEvent::from(message));
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum IncomingEvent {
    Join { connection_id: u64 },
    Message { message: String },
}

impl From<String> for IncomingEvent {
    fn from(message: String) -> Self {
        Self::Message { message }
    }
}

pub enum OutgoingEvent {
    Joined {
        connection_id: u64,
        field: cark_common::Field,
    },
    Message {
        message: String,
    },
}

impl From<String> for OutgoingEvent {
    fn from(message: String) -> Self {
        Self::Message { message }
    }
}
