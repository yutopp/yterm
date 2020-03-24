use crate::ui_gtk::im;
use std::rc::Rc;

pub struct UIShared {
    pub window: gtk::ApplicationWindow,

    pub im: Rc<im::IM>,
}
