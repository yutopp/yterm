use std::cell::RefCell;
use std::rc::Rc;

pub struct IM {
    pub context: gtk::IMMulticontext,

    hooks: RefCell<Option<Rc<IMEventHooksRecords>>>,
}

impl IM {
    pub fn new() -> Self {
        let im_context = gtk::IMMulticontext::new();

        Self {
            context: im_context,
            hooks: RefCell::new(None),
        }
    }

    pub fn set_active_hooks(&self, other: Option<Rc<IMEventHooksRecords>>) {
        let mut records = self.hooks.borrow_mut();
        *records = other;
    }

    pub fn call_connect(&self, text: &str) {
        let hooks = self.hooks.borrow().clone();
        if let Some(hooks) = hooks {
            hooks.connect.as_ref()(text);
        }
    }
}

pub struct IMEventHooksRecords {
    pub connect: Box<dyn Fn(&str) -> ()>,
}
