use crate::message::event::Event;

#[derive(Clone, Debug)]
pub enum Message {
    Cast(Event),
    Call(u64, Event),
}
