use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use gio::prelude::*;
use gtk::prelude::*;

use crate::conn::Conn;
use crate::message::Event;
use crate::ui_gtk::event_bridge;
use crate::ui_gtk::im;
use crate::ui_gtk::state;
use crate::ui_gtk::terminal as ui_terminal;

pub struct Shared<C> {
    pub conn: C,
}

struct State {
    columns: i32,
    rows: i32,
    font_width: i32,
    font_height: i32,
}

pub struct UI<C> {
    shared: Shared<C>,
}

impl<C> UI<C>
where
    C: Conn,
{
    pub fn new(shared: Shared<C>) -> Self {
        Self { shared }
    }

    pub fn run(self) {
        let application = gtk::Application::new(Some("net.yutopp.yterm"), Default::default())
            .expect("failed to initialize GTK application");

        let m: HashMap<u64, Arc<RefCell<ui_terminal::UI>>> = HashMap::new();
        let windows = Rc::new(RefCell::new(m));

        let Shared { conn } = self.shared;

        let (cast_tx, cast_rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let bridge = event_bridge::EventBridge::new(conn, cast_tx);
        cast_rx.attach(None, {
            let windows = windows.clone();
            move |event| {
                println!("broadcast: {:?}", event);

                match event {
                    Event::WindowWrite(id, content) => {
                        if let Some(ui) = windows.borrow_mut().get(&id) {
                            ui.borrow_mut().handle_message(content);
                        }
                    }
                    _ => (),
                }

                glib::Continue(true)
            }
        });

        application.connect_activate({
            let windows = windows.clone();
            move |app| {
                let im = Rc::new(im::IM::new());
                im.context.connect_commit({
                    let im = im.clone();
                    move |_im_context, text| {
                        im.call_connect(text);
                    }
                });

                let state = Arc::new(State {
                    columns: 40,
                    rows: 16,
                    font_width: 20,
                    font_height: 26,
                });

                let window = gtk::ApplicationWindow::new(app);
                window.set_title("yterm");
                window.set_default_size(
                    state.columns * state.font_width,
                    state.rows * state.font_height,
                );

                let ui_shared = state::UIShared {
                    window: window.clone(),

                    bridge: bridge.clone(),
                    im,
                };

                // Test
                bridge.cast(Event::Testing).unwrap();
                bridge.call(Event::Testing).unwrap();

                //shared.conn.a;

                //let terminal = terminal::Terminal::new(shared.clone());
                let id = 0;
                let terminal_area = ui_terminal::UI::new(ui_shared, id);
                //
                window.add(&terminal_area.borrow().widget);
                window.set_focus(Some(&terminal_area.borrow().widget));

                windows.borrow_mut().insert(id, terminal_area);

                window.show_all();
            }
        });

        application.run(&[]);
    }
}
