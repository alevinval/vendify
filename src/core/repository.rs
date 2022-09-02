use anyhow::format_err;

use super::utils::GitOps;
use crate::core::utils::PathIterator;
use crate::core::utils::WalkdirPathIterator;
mod repository_path_factory;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;
use git2::Oid;
use repository_path_factory::RepositoryPathFactory;

use crate::core::Dependency;

pub struct Repository {
    pub path: PathBuf,
    path_iterator: Box<dyn PathIterator>,
    git: GitOps,
}

impl Repository {
    pub fn new<P: AsRef<Path>>(cache: P, dep: &Dependency) -> Self {
        let git = GitOps {};
        let repo_path = RepositoryPathFactory::create(dep, cache);
        let path_iterator = WalkdirPathIterator::new(&repo_path);
        Repository {
            path: repo_path,
            path_iterator: Box::new(path_iterator),
            git,
        }
    }

    pub fn iter(&self) -> Box<dyn Iterator<Item = PathBuf>> {
        self.path_iterator.iter()
    }

    pub fn checkout(&self, refname: &str) -> Result<()> {
        self.git.checkout(&self.path, refname)
    }

    pub fn fetch(&self, refname: &str) -> Result<()> {
        self.git.fetch(&self.path, refname)
    }

    pub fn reset(&self, refname: &str) -> Result<()> {
        self.git.reset(&self.path, refname)
    }

    pub fn get_current_refname(&self) -> Result<Oid> {
        self.git.get_current_refname(&self.path)
    }

    pub fn ensure_repository(&self, dep: &Dependency) -> Result<()> {
        let result = self.git.open_or_clone(&dep.url, &dep.refname, &self.path);

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(format_err!("cannot open repository: {}", err)),
        }
    }
}
