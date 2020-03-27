use crate::logic::state::Shared;
use crate::message::{Message, Event};

pub struct Window {
    init: Shared,
    id: u64,
}

impl Window {
    pub fn new(init: Shared, id: u64) -> Self {
        Self {
            init,
            id,
        }
    }

    pub async fn write(&self, s: String) {
        let mut conn_sender = self.init.conn_sender.clone();
        let id = self.id;

        conn_sender.send(Message::Cast(Event::WindowWrite(id, s))).await;
    }
}
