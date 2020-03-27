use tokio::sync::mpsc;

use yterm_lib::conn::channel::ChannelConn;

pub struct LocalConnector {
    server_conn: Option<ChannelConn>,
    client_conn: Option<ChannelConn>,
}

impl LocalConnector {
    pub fn new() -> Self {
        let (server_tx, server_rx) = mpsc::channel(100);
        let (client_tx, client_rx) = mpsc::channel(100);
        Self {
            server_conn: Some(ChannelConn::new(client_tx, server_rx)),
            client_conn: Some(ChannelConn::new(server_tx, client_rx)),
        }
    }

    pub fn server_conn(&mut self) -> Option<ChannelConn> {
        let c = self.server_conn.take();
        return c;
    }

    pub fn client_attach_to_local_server(&mut self) -> Option<ChannelConn> {
        let c = self.client_conn.take();
        return c;
    }
}
