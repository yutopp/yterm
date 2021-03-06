use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use gtk::prelude::*;
//use gio::prelude::*;
use gtk::{DrawingArea, DrawingAreaBuilder};
use yterm_lib::message::Event;

use crate::im;
use crate::state::UIShared;

pub struct UI {
    pub widget: DrawingArea,

    id: u64,
    ui_shared: UIShared,
    //handle: Handle,
    columns: i32,
    rows: i32,
    font_width: i32,
    font_height: i32,

    texts: Vec<char>,
}

impl UI {
    pub fn new(ui_shared: UIShared, id: u64) -> Arc<RefCell<Self>> {
        let drawing = DrawingAreaBuilder::new()
            .can_focus(true)
            .events(gdk::EventMask::KEY_PRESS_MASK)
            .events(gdk::EventMask::KEY_RELEASE_MASK)
            .build();

        //let (term_tx, term_rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        //let handle = t.start_thread(term_tx);

        let ui = Arc::new(RefCell::new(Self {
            widget: drawing,

            id,
            ui_shared: ui_shared,

            columns: 40,
            rows: 16,
            font_width: 20,
            font_height: 26,

            texts: vec![],
            //handle: handle,
        }));

        ui.borrow().change_ui_size();

        let im_hooks = {
            let ui = ui.clone();
            Rc::new(im::IMEventHooksRecords {
                connect: Box::new(move |text: &str| {
                    ui.borrow_mut().commit_text(text);
                }),
            })
        };

        let widget = ui.borrow().widget.clone();
        let im = ui.borrow().ui_shared.im.clone();

        widget.connect_realize({
            let im_context = im.context.clone();
            move |widget| {
                im_context.set_client_window(widget.get_window().as_ref());
            }
        });
        widget.connect_unrealize({
            let im_context = im.context.clone();
            move |widget| {
                im_context.set_client_window(widget.get_window().as_ref());
            }
        });
        widget.connect_focus_in_event({
            let im = im.clone();
            move |_widget, _event| {
                println!("focus_in");

                let im_context = im.context.clone();
                im_context.focus_in();
                im.set_active_hooks(Some(im_hooks.clone()));

                Inhibit(false)
            }
        });
        widget.connect_focus_out_event({
            let im = im.clone();
            move |_widget, _event| {
                println!("focus_out");

                let im_context = im.context.clone();
                im.set_active_hooks(None);
                im_context.focus_out();

                Inhibit(false)
            }
        });

        widget.connect_key_press_event({
            let im = im.clone();
            let ui = ui.clone();
            move |_widget, event| {
                let im_context = im.context.clone();
                if im_context.filter_keypress(event) {
                    return Inhibit(true);
                }

                let keyval = event.get_keyval();
                println!("-> {:?} ({:?})", event, keyval);

                if keyval == 65293 {
                    ui.borrow_mut().on_key('\r');
                } else {
                    //ui.borrow_mut().on_key(keyval as char);
                }

                Inhibit(true)
            }
        });

        widget.connect_draw({
            let ui = ui.clone();
            move |widget, cr| ui.borrow_mut().draw_editor(widget, cr)
        });

        ui
    }

    fn change_ui_size(&self) {
        //let state = &self.handle.state;

        self.widget
            .set_size_request(self.columns * self.font_width, self.rows * self.font_height);
    }

    fn commit_text(&mut self, text: &str) {
        println!("-> {}", text);
        self.on_keys(text);
    }

    fn draw_editor(&self, widget: &DrawingArea, cr: &cairo::Context) -> Inhibit {
        let width = widget.get_allocated_width();
        let height = widget.get_allocated_height();

        cr.set_source_rgb(0.8, 0.8, 0.8);
        cr.rectangle(0.0, 0.0, width.into(), height.into());
        cr.fill();

        cr.select_font_face(
            "IPAGothic",
            cairo::FontSlant::Normal,
            cairo::FontWeight::Normal,
        );
        cr.set_font_size(16.0);
        cr.set_source_rgb(0.0, 0.0, 0.0);

        let mut pos_x = 0.0;
        let mut pos_y = 0.0;
        let mut buf = [0; 4];

        for c in self.texts.iter() {
            match c {
                '\n' => {
                    pos_x = 0.0;
                    pos_y += 20.0;
                }
                _ => {
                    let str = c.encode_utf8(&mut buf);

                    let ext = cr.text_extents(str);
                    cr.move_to(pos_x - ext.x_bearing, pos_y - ext.y_bearing);
                    cr.show_text(str);

                    pos_x += ext.width + ext.x_advance;
                }
            }
        }

        Inhibit(true)
    }

    pub fn handle_message(&mut self, seq: String) {
        for c in seq.chars() {
            println!("-> {:?}", c);

            self.texts.push(c);
        }

        self.widget.queue_draw();
    }

    pub fn on_key(&mut self, ch: char) {
        self.ui_shared.bridge.cast(Event::KeyInput(self.id, ch));

        //self.handle.send_char(ch);
    }

    pub fn on_keys(&mut self, text: &str) {
        for ch in text.chars() {
            self.on_key(ch);
        }
    }
}
