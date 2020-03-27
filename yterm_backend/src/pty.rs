use std::fs::File;
use std::os::unix::io::{AsRawFd, FromRawFd};

use libc::{grantpt, open, posix_openpt, ptsname, unlockpt, O_NOCTTY, O_RDWR};

pub struct Master {
    pub handle: File,
}

impl Master {
    pub unsafe fn open() -> Self {
        let fd = posix_openpt(O_RDWR);
        if fd < 0 {
            panic!("Failed to posix_openpt"); // TODO
        }

        let m = Self {
            handle: FromRawFd::from_raw_fd(fd),
        };

        if grantpt(fd) == -1 {
            panic!("Failed to grantpt"); // TODO
        }
        if unlockpt(fd) == -1 {
            panic!("Failed to unlockpt"); // TODO
        };

        m
    }

    pub unsafe fn open_slave(&self) -> Slave {
        let fd = self.handle.as_raw_fd();

        let slave_name = ptsname(fd);
        let slave_fd = open(slave_name, O_RDWR | O_NOCTTY);
        if slave_fd < 0 {
            panic!("Failed to open"); // TODO
        }

        Slave {
            handle: FromRawFd::from_raw_fd(slave_fd),
        }
    }
}

pub struct Slave {
    pub handle: File,
}
