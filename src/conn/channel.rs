use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::conn::conn;
use crate::message::Message;

pub struct ChannelConn {
    tx: mpsc::Sender<Message>,
    rx: mpsc::Receiver<Message>,
}

impl ChannelConn {
    pub fn new(tx: mpsc::Sender<Message>, rx: mpsc::Receiver<Message>) -> Self {
        Self { tx, rx }
    }
}

impl conn::Conn for ChannelConn {
    type Sender = ChannelConnSender;
    type Receiver = ChannelConnReceiver;

    fn split(self) -> (Self::Sender, ChannelConnReceiver) {
        let sender = ChannelConnSender(self.tx);
        (sender, ChannelConnReceiver(self.rx))
    }
}

#[derive(Clone)]
pub struct ChannelConnSender(mpsc::Sender<Message>);

#[async_trait]
impl conn::ConnSender for ChannelConnSender {
    fn try_send(&self, msg: Message) {
        let mut tx = self.0.clone();
        let _ = tx.try_send(msg);
    }

    async fn send(&self, msg: Message) {
        let mut tx = self.0.clone();
        let _ = tx.send(msg).await;
    }
}

pub struct ChannelConnReceiver(mpsc::Receiver<Message>);

#[async_trait]
impl conn::ConnReceiver for ChannelConnReceiver {
    async fn recv(&mut self) -> Option<Message> {
        self.0.recv().await
    }
}
