use std::path::Path;
use std::path::PathBuf;

use anyhow::format_err;
use anyhow::Result;

use self::git::Git;
use crate::deps::Dependency;

mod git;

pub struct Repository {
    path: PathBuf,
}

impl Repository {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_owned(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn checkout(&self, refname: &str) -> Result<()> {
        Git::checkout(&self.path, refname)
    }

    pub fn fetch(&self, refname: &str) -> Result<()> {
        Git::fetch(&self.path, refname)
    }

    pub fn reset(&self, refname: &str) -> Result<()> {
        Git::reset(&self.path, refname)
    }

    pub fn get_current_refname(&self) -> Result<String> {
        Git::get_current_refname(&self.path).map(|oid| oid.to_string())
    }

    pub fn ensure(self, dep: &Dependency) -> Result<Self> {
        let result = Git::open_or_clone(&dep.url, &dep.refname, &self.path);

        match result {
            Ok(_) => Ok(self),
            Err(err) => Err(format_err!("cannot open repository: {}", err)),
        }
    }
}
