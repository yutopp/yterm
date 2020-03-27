use std::sync::Arc;
use async_trait::async_trait;

use crate::conn::channel;
use crate::message::{Message};

pub trait Conn {
    type Sender: ConnSender + Send + Sync + 'static;
    type Receiver: ConnReceiver;

    fn split(self) -> (Self::Sender, channel::ChannelConnReceiver);
}

pub type ArcConnSender = Arc<dyn ConnSender + Send + Sync>;

#[async_trait]
pub trait ConnSender {
    fn try_send(&self, msg: Message);
    async fn send(&self, msg: Message);
}

#[async_trait]
pub trait ConnReceiver {
    async fn recv(&mut self) -> Option<Message>;
}
