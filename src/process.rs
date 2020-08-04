
use std::sync::{Arc, Mutex};

use if_let_return::*;

use crate::display;
use crate::errors::{AppError, AppResultU};



#[derive(Clone)]
pub struct Process {
    pid: Arc<Mutex<Option<u32>>>,
    signal: i32,
}


impl Process {
    pub fn is_empty(&self) -> bool {
        self.pid.lock().unwrap().is_none()
    }

    pub fn new(signal: i32) -> Self {
        Process { pid: Arc::new(Mutex::new(None)), signal }
    }

    pub fn release(&self) {
        let _ = self.pid.lock().unwrap().take();
    }

    pub fn set(&self, new_pid: u32) {
        let mut pid = self.pid.lock().unwrap();
        *pid = Some(new_pid);
    }

    pub fn terminate(&self) -> AppResultU {
        if_let_some!(pid = (*self.pid.lock().unwrap()).take(), Ok(()));

        display::killing(pid);
        unsafe {
            let mut status = 1;
            let pid = pid as i32;
            let err = libc::kill(-pid, self.signal);
            if err != 0 {
                return Err(AppError::Errno(errno::errno()));
            }
            libc::waitpid(pid as i32, &mut status, 0);
        }

        Ok(())
    }
}
