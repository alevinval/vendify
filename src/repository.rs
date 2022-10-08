use std::path::Path;
use std::path::PathBuf;

use anyhow::format_err;
use anyhow::Result;
use git2::Oid;

use self::git::Git;
use crate::dependency::Dependency;

mod git;

pub struct Repository {
    path: PathBuf,
    git: Git,
}

impl Repository {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let git = Git {};
        Repository {
            path: path.as_ref().to_owned(),
            git,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn checkout(&self, refname: &str) -> Result<()> {
        Git::checkout(&self.path, refname)
    }

    pub fn fetch(&self, refname: &str) -> Result<()> {
        self.git.fetch(&self.path, refname)
    }

    pub fn reset(&self, refname: &str) -> Result<()> {
        Git::reset(&self.path, refname)
    }

    pub fn get_current_refname(&self) -> Result<Oid> {
        Git::get_current_refname(&self.path)
    }

    pub fn ensure(self, dep: &Dependency) -> Result<Self> {
        let result = self.git.open_or_clone(&dep.url, &dep.refname, &self.path);

        match result {
            Ok(_) => Ok(self),
            Err(err) => Err(format_err!("cannot open repository: {}", err)),
        }
    }
}
