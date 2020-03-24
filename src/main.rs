extern crate yterm;

use std::rc::Rc;
use std::thread;
use std::time::Duration;

fn main() {
    let rt = tokio::runtime::Builder::new()
        .threaded_scheduler()
        .enable_all()
        .build()
        .unwrap();

    // UI Thread
    let ui_th = thread::spawn({
        let handle = rt.handle().clone();
        let shared = yterm::logic::state::Shared { rt: handle };
        move || {
            let ui = yterm::ui_gtk::app::UI::new(Rc::new(shared));
            ui.run();
        }
    });
    ui_th.join().unwrap();

    rt.shutdown_timeout(Duration::from_millis(100));
}
