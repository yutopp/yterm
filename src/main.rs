extern crate yterm_backend;
extern crate yterm_frontend_gtk3;

mod local_connector;

use std::thread;
use std::time::Duration;
use tokio::task;

fn main() {
    let rt = tokio::runtime::Builder::new()
        .threaded_scheduler()
        .enable_all()
        .build()
        .unwrap();

    let mut connector = local_connector::LocalConnector::new();

    // Backend
    rt.spawn({
        let handle = rt.handle().clone();
        let app = yterm_backend::app::App {
            rt: handle,
            conn: connector.server_conn().expect("Should gettable"),
        };

        async { app.run_main_loop().await }
    });

    // UI Thread
    let ui_th = thread::spawn(|| {
        let mut rt = tokio::runtime::Builder::new()
            .threaded_scheduler()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async move {
            let conn = connector
                .client_attach_to_local_server()
                .expect("Should attachable");
            let shared = yterm_frontend_gtk3::app::Shared { conn };
            let join = task::spawn_blocking(move || {
                let ui = yterm_frontend_gtk3::app::UI::new(shared);
                ui.run();
            });
            join.await.unwrap();
        });
    });
    ui_th.join().unwrap();

    rt.shutdown_timeout(Duration::from_millis(100));
}
