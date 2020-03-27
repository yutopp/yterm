use std::ffi::CString;
use std::os::unix::io::AsRawFd;
use std::sync::Arc;

use crate::logic::pty;
use crate::logic::window;
use crate::logic::state::Shared;

#[derive(Debug)]
pub enum Event {
    Terminal(Vec<u8>),
}

pub struct Terminal {
    master: pty::Master,
    slave: pty::Slave,

    shared: Shared,
    state: State,
}

pub struct State {
    pub columns: i32,
    pub rows: i32,
    pub font_width: i32,
    pub font_height: i32,

    pub texts: Vec<char>,
}

impl Terminal {
    pub fn new(shared: Shared) -> Self {
        let master = unsafe { pty::Master::open() };
        let slave = unsafe { master.open_slave() };

        Self {
            master,
            slave,

            shared,
            state: State {
                columns: 40,
                rows: 16,
                font_width: 20,
                font_height: 26,

                texts: vec![],
            },
        }
    }

    pub fn start_thread(self, win: Arc<window::Window>) -> Handle {
        self.set_winsize();

        let pid = unsafe { libc::fork() };
        if pid == -1 {
            panic!("failed to fork");
        } else if pid == 0 {
            // child process!
            unsafe { self.into_child_process() };
            unreachable!()
        } else if pid > 0 {
            // parent process
            return self.parent_process(win);
        }

        unreachable!();
    }

    fn set_winsize(&self) {
        let ws = libc::winsize {
            ws_col: self.state.columns as u16,
            ws_row: self.state.rows as u16,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };

        let fd = self.master.handle.as_raw_fd();
        unsafe {
            libc::ioctl(fd, libc::TIOCSWINSZ, &ws); // TODO: error
        }
    }

    unsafe fn into_child_process(self) {
        drop(self.master);

        if libc::setsid() == -1 {
            panic!("{}", std::io::Error::last_os_error());
        }

        {
            let fd = self.slave.handle.as_raw_fd();

            if libc::ioctl(fd, libc::TIOCSCTTY, 0) == -1 {
                panic!("{}", std::io::Error::last_os_error());
            }

            if libc::dup2(fd, libc::STDIN_FILENO) == -1 {
                panic!("{}", std::io::Error::last_os_error());
            }
            if libc::dup2(fd, libc::STDOUT_FILENO) == -1 {
                panic!("{}", std::io::Error::last_os_error());
            }
            if libc::dup2(fd, libc::STDERR_FILENO) == -1 {
                panic!("{}", std::io::Error::last_os_error());
            }
        }
        drop(self.slave);

        std::env::set_var("TERM", "term-256color");
        std::env::set_var("COLORTERM", "truecolor");
        std::env::set_var("COLUMNS", &self.state.columns.to_string());
        std::env::set_var("LINES", &self.state.rows.to_string());

        let path = CString::new("/usr/bin/sh").expect("Failed to CString::new");
        let mut args = vec![];
        args.push(std::ptr::null());
        if libc::execvp(path.as_ptr(), args.as_ptr()) == -1 {
            panic!("{}", std::io::Error::last_os_error());
        }
        unreachable!();
    }

    fn parent_process(self, win: Arc<window::Window>) -> Handle {
        drop(self.slave);

        let reader = self.master.handle;
        let writer = reader.try_clone().unwrap();

        self.shared.rt.spawn({
            let win = win.clone();
            let mut reader = tokio::fs::File::from_std(reader);
            async move {
                use tokio::prelude::*;

                const BUFFER_SIZE: usize = 32 * 1024;
                let mut buf = [0u8; BUFFER_SIZE];
                loop {
                    if let Ok(n) = reader.read(&mut buf[..]).await {
                        let s = String::from_utf8(buf[..n].into()).unwrap();
                        println!("{:?}",s);

                        win.write(s).await;
                        //tx.send(Event::Terminal(buf[..n].into())).unwrap();
                    }
                }
            }
        });

        Handle {
            win,
            state: self.state,
            writer,
        }
    }
}

pub struct Handle {
    win: Arc<window::Window>,
    pub state: State,

    writer: std::fs::File,
}

impl Handle {
    pub fn recv_char(&mut self, c: char) {
        self.state.texts.push(c)
    }

    pub fn send_char(&mut self, c: char) {
        use std::io::Write;

        let mut buf = [0; 4];
        let str = c.encode_utf8(&mut buf);
        self.writer.write(str.as_bytes());
    }
}
