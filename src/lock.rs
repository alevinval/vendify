use std::fs::File;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

use anyhow::Result;
use log::error;

pub struct Lock {
    path: PathBuf,
    file: Option<File>,
    warn: Option<String>,
    after: Option<Duration>,
}

impl Lock {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            file: None,
            warn: None,
            after: None,
        }
    }

    pub fn with_warn(mut self, warn: impl Into<String>, after: Duration) -> Self {
        self.warn = Some(warn.into());
        self.after = Some(after);
        self
    }

    pub fn acquire(&mut self) -> Result<()> {
        if self.file.is_none() {
            self.file = Some(File::create(&self.path)?);
        }
        let file = self.file.as_ref().unwrap();
        thread::scope(|s| -> Result<()> {
            let (sender, receiver): (Sender<Result<()>>, Receiver<Result<()>>) = mpsc::channel();
            s.spawn(move || {
                let result = unix::exclusive_lock(file);
                sender.send(result)
            });
            if let Some(after) = self.after {
                if receiver.recv_timeout(after).is_ok() {
                    return Ok(());
                }
                error!("{}", self.warn.as_ref().unwrap());
            }
            match receiver.recv() {
                Ok(result) => result,
                Err(err) => Err(err.into()),
            }
        })
    }
}

mod unix {
    use std::fs::File;
    use std::os::unix::prelude::AsRawFd;

    use anyhow::format_err;
    use anyhow::Result;

    pub fn exclusive_lock(file: &File) -> Result<()> {
        flock(file, libc::LOCK_EX)
    }

    fn flock(file: &File, flag: libc::c_int) -> Result<()> {
        let ret = unsafe { libc::flock(file.as_raw_fd(), flag) };
        if ret < 0 {
            Err(format_err!("cannot lock file"))
        } else {
            Ok(())
        }
    }
}
