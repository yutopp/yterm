use crate::logic::app;

pub struct Shared {
    pub rt: tokio::runtime::Handle,
    pub conn_sender: app::ConnSender,
}
