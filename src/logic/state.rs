use crate::conn::{ArcConnSender};

#[derive(Clone)]
pub struct Shared {
    pub rt: tokio::runtime::Handle,
    pub conn_sender: ArcConnSender,
}
