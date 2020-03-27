use crate::logic::state::Shared;
use crate::message::{Event, Message};

pub struct Window {
    init: Shared,
    id: u64,
}

impl Window {
    pub fn new(init: Shared, id: u64) -> Self {
        Self { init, id }
    }

    pub async fn write(&self, s: String) {
        let conn_sender = &self.init.conn_sender;
        let id = self.id;

        conn_sender
            .send(Message::Cast(Event::WindowWrite(id, s)))
            .await;
    }
}
