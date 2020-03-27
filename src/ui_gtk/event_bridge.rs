use std::rc::Rc;
use std::sync::Arc;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::RwLock;

use tokio::task;

use crate::logic::app;

pub struct EventBridge {
    pub conn_sender: app::ConnSender,
    id_map: Arc<RwLock<EventRelation>>,
}

// [Backend]
//   | (Conn)
// [Bridge] <-> (tokio)
//   |
// [U I]

impl EventBridge {
    pub fn new(
        conn: app::Conn,
        ui_cast_tx: glib::Sender<app::Event>,
    ) -> Rc<Self> {
        let (conn_sender, mut conn_receiver) = conn.split(); // backend

        //let (brg_tx, brg_rx) = mpsc::sync_channel(200);

        let id_map = Arc::new(RwLock::new(EventRelation::new()));

        // a task to receive messages from the backend and send them to the UI.
        let join_recv = task::spawn({
            let id_map = id_map.clone();
            let ui_cast_tx = ui_cast_tx.clone();
            async move {
                while let Some(response) = conn_receiver.recv().await {
                    eprintln!("Tx = {:?}", response.clone());
                     let join = task::spawn_blocking({
                        let id_map = id_map.clone();
                        let ui_cast_tx = ui_cast_tx.clone();
                        move || {
                            match response {
                                app::Message::Cast(ev) => {
                                    eprintln!("Cast = {:?}", ev);
                                    ui_cast_tx.send(ev).unwrap();
                                }
                                app::Message::Call(id, ev) => {
                                    eprintln!("Call = {:?}", ev);
                                    let mut map_ref = id_map.write().unwrap();
                                    map_ref.response(&id, ev);
                                }
                            }
                        }
                    });
                    let handle = join.await;
                }
            }
        });

        Rc::new(Self {
            conn_sender,
            id_map: id_map,
        })
    }

    pub fn cast(&self, e: app::Event) -> Result<(), ()> {
        let mut sender = self.conn_sender.clone();
        sender.try_send(app::Message::Cast(e));

        Ok(())
    }

    pub fn call(&self, e: app::Event) -> Result<(), ()> {
        let (tx, rx) = mpsc::sync_channel(1);
        let cur = {
            let id_map = self.id_map.clone();
            let mut map_ref = id_map.write().unwrap();
            map_ref.register(tx)
        };

        let mut sender = self.conn_sender.clone();
        sender.try_send(app::Message::Call(cur, e));

        let event = rx.recv().unwrap();
        println!("received: {:?}", event);

        Ok(())
    }
}

struct EventRelation {
    uniq: u64,
    msg_recv_tx_map: HashMap<u64, Box<dyn Fn(app::Event) -> () + Send + Sync>>,
}

impl EventRelation {
    fn new() -> Self {
        Self {
            uniq: 0,
            msg_recv_tx_map: HashMap::default(),
        }
    }

    fn register(&mut self, tx: mpsc::SyncSender<app::Event>) -> u64 {
        let cur = self.uniq;
        self.uniq+=1;
        self.msg_recv_tx_map.insert(cur, Box::new(move |ev| {
                println!("callback");
                tx.send(ev).unwrap();
            ()
        }));

        cur
    }

    fn response(&mut self, id: &u64, e: app::Event) -> Option<app::Event> {
        let cb_opt = self.msg_recv_tx_map.remove(&id);
        if let Some(cb) = cb_opt {
            eprintln!("Tx = {:?}", e);
            cb(e);
            None
        } else {
            // Cannot find callback
            Some(e)
        }
    }
}
