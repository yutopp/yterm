use std::rc::Rc;

use crate::event_bridge;
use crate::im;

pub struct UIShared {
    pub window: gtk::ApplicationWindow,

    pub bridge: Rc<event_bridge::EventBridge>,
    pub im: Rc<im::IM>,
}
