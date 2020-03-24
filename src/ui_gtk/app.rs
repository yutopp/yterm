use std::rc::Rc;
use std::sync::Arc;

use gio::prelude::*;
use gtk::prelude::*;

use crate::logic::state::Shared;
use crate::logic::terminal;
use crate::ui_gtk::im;
use crate::ui_gtk::state;
use crate::ui_gtk::terminal as ui_terminal;

struct State {
    columns: i32,
    rows: i32,
    font_width: i32,
    font_height: i32,
}

pub struct UI {
    shared: Rc<Shared>,
}

impl UI {
    pub fn new(shared: Rc<Shared>) -> Self {
        Self { shared }
    }

    pub fn run(self) {
        let application = gtk::Application::new(Some("net.yutopp.yterm"), Default::default())
            .expect("failed to initialize GTK application");

        application.connect_activate({
            let shared = self.shared.clone();

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

                    im,
                };

                let terminal = terminal::Terminal::new(shared.clone());
                let terminal_area = ui_terminal::UI::new(ui_shared, terminal);

                window.add(&terminal_area.borrow().widget);
                window.set_focus(Some(&terminal_area.borrow().widget));

                window.show_all();
            }
        });

        application.run(&[]);
    }
}
