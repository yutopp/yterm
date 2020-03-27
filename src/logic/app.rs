use std::sync::Arc;
use std::collections::HashMap;

use tokio::task;

use crate::message::{Message, Event};
use crate::logic::window;
use crate::logic::terminal;
use crate::logic::state::Shared;

pub use crate::conn::{ArcConnSender, ConnReceiver};
use crate::conn::conn::Conn;

pub struct App<C> where C: Conn {
    pub rt: tokio::runtime::Handle,
    pub conn: C,
}

impl<C> App<C> where C: Conn {
    pub async fn run_main_loop(self) {
        let (conn_sender, mut conn_receiver) = self.conn.split();

        let mut terminals = HashMap::new();
        let conn_sender = Arc::new(conn_sender);
        let shared = Shared {
            rt: self.rt.clone(),
            conn_sender,
        };

        loop {
            while let Some(request) = conn_receiver.recv().await {
                println!("main: request: {:?}", request);

                match request {
                    Message::Call(id, Event::Testing) => {
                        let join = task::spawn_blocking({
                            let init = shared.clone();
                            move || {
                                let window = Arc::new(window::Window::new(init.clone(), id));

                                let terminal = terminal::Terminal::new(init.clone());
                                let handle = terminal.start_thread(window.clone());
                                (window, handle)
                            }
                        });
                        if let Ok((window, handle)) = join.await {
                            terminals.insert(id, (window, handle));
                        }

                        shared.conn_sender.send(Message::Call(id, Event::Testing)).await;
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
