use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::task;

use crate::logic::window;
use crate::logic::terminal;
use crate::logic::state::Shared;

pub struct App {
    pub rt: tokio::runtime::Handle,
    pub conn: Conn,
}

impl App {
    pub async fn run_main_loop(self) {
        let (mut conn_sender, mut conn_receiver) = self.conn.split();

        let mut terminals = HashMap::new();

        loop {
            while let Some(request) = conn_receiver.recv().await {
                println!("main: request: {:?}", request);

                match request {
                    Message::Call(id, Event::Testing) => {
                        let join = task::spawn_blocking({
                            let init = Shared {
                                rt: self.rt.clone(),
                                conn_sender: conn_sender.clone(),
                            };
                            let init2 = Shared {
                                rt: self.rt.clone(),
                                conn_sender: conn_sender.clone(),
                            };
                            move || {
                                let window = Arc::new(window::Window::new(init2, id));

                                let terminal = terminal::Terminal::new(init);
                                let handle = terminal.start_thread(window.clone());
                                (window, handle)
                            }
                        });
                        if let Ok((window, handle)) = join.await {
                            terminals.insert(id, (window, handle));
                        }

                        conn_sender.send(Message::Call(id, Event::Testing)).await;
                    },
                    Message::Cast(Event::KeyInput(id, ch)) => {
                        if let Some((_window, handle)) = terminals.get_mut(&id) {
                            handle.send_char(ch);
                        }
                    },
                    _ => {
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    Cast(Event),
    Call(u64, Event),
}

#[derive(Clone, Debug)]
pub enum Event {
    Testing,
    KeyInput(u64, char),
    WindowWrite(u64, String),
}

pub struct Connector {
    server_conn: Option<Box<Conn>>,
    client_conn: Option<Box<Conn>>,
}

impl Connector {
    pub fn new() -> Self {
        let (server_tx, server_rx) = mpsc::channel(100);
        let (client_tx, client_rx) = mpsc::channel(100);
        Self {
            server_conn: Some(Box::new(Conn::new(client_tx, server_rx))),
            client_conn: Some(Box::new(Conn::new(server_tx, client_rx))),
        }
    }

    pub fn server_conn(&mut self) -> Option<Conn> {
        let c = self.server_conn.take();
        return c.map(|p| *p)
    }

    pub fn client_attach_to_local_server(&mut self) -> Option<Conn> {
        let c = self.client_conn.take();
        return c.map(|p| *p)
    }
}

pub struct Conn {
    tx: mpsc::Sender<Message>,
    rx: mpsc::Receiver<Message>,
}

impl Conn {
    pub fn new(tx: mpsc::Sender<Message>, rx: mpsc::Receiver<Message>) -> Self {
        Self { tx, rx }
    }

    pub fn split(self) -> (ConnSender, ConnReceiver) {
        (ConnSender(self.tx), ConnReceiver(self.rx))
    }
}

#[derive(Clone)]
pub struct ConnSender(mpsc::Sender<Message>);

impl ConnSender {
    pub fn try_send(&mut self, ev: Message) {
        let _ = self.0.try_send(ev);
    }

    pub async fn send(&mut self, ev: Message) {
        let _ = self.0.send(ev).await;
    }
}

pub struct ConnReceiver(mpsc::Receiver<Message>);

impl ConnReceiver {
    pub async fn recv(&mut self) -> Option<Message> {
        self.0.recv().await
    }
}
