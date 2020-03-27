use crate::ui_gtk::im;
use crate::ui_gtk::event_bridge;
use std::rc::Rc;

pub struct UIShared {
    pub window: gtk::ApplicationWindow,

    pub bridge: Rc<event_bridge::EventBridge>,
    pub im: Rc<im::IM>,
}
