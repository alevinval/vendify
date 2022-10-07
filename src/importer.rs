use std::path::Path;

use anyhow::Result;
use log::debug;
use log::info;

use self::collector::Collector;
use self::selector::Selector;
use crate::dependency::Dependency;
use crate::dependency::Lock;
use crate::repository::Repository;
use crate::spec::Spec;

mod collector;
mod selector;

pub struct Importer<'a> {
    dependency: &'a Dependency,
    dependency_lock: Option<&'a Lock>,
    repository: &'a Repository,
    collector: Collector,
}

impl<'a> Importer<'a> {
    pub fn new(
        spec: &'a Spec,
        dependency: &'a Dependency,
        dependency_lock: Option<&'a Lock>,
        repository: &'a Repository,
    ) -> Self {
        Importer {
            dependency,
            dependency_lock,
            repository,
            collector: Collector::new(
                &repository.path(),
                &Path::new(&spec.vendor),
                Selector::new(spec, dependency),
            ),
        }
    }

    /// Install copies the files of the dependency into the vendor folder.
    /// It respects the dependency lock, when passed.
    pub fn install(&self) -> Result<Lock> {
        let refname = self.get_locked_refname();

        info!("installing {}@{}", self.dependency.url, refname);
        self.repository.fetch(&self.dependency.refname)?;
        self.repository.checkout(refname)?;
        self.import()
    }

    /// Update fetches latest changes from the git remote, against the
    /// reference. Then it installs the dependency. This will ignore the
    /// lock file and generate a new lock with the updated reference.
    pub fn update(&self) -> Result<Lock> {
        let refname = self.dependency.refname.as_str();

        info!("updating {}@{}", self.dependency.url, refname);
        self.repository.fetch(refname)?;
        self.repository.reset(refname)?;
        self.import()
    }

    fn import(&self) -> Result<Lock> {
        self.copy_files()?;
        let locked = self.get_locked_dependency()?;
        info!("\t🔒 {}", locked.refname);
        Ok(locked)
    }

    fn copy_files(&self) -> Result<()> {
        for collected in self.collector.iter() {
            debug!(
                "\t.../{} -> {}",
                collected.src_rel.display(),
                collected.dst.display()
            );
            collected.copy()?;
        }
        Ok(())
    }

    fn get_locked_refname(&self) -> &str {
        match self.dependency_lock {
            Some(it) => &it.refname,
            None => &self.dependency.refname,
        }
    }

    fn get_locked_dependency(&self) -> Result<Lock> {
        let refname = self.repository.get_current_refname()?;
        Ok(Lock {
            url: self.dependency.url.clone(),
            refname: refname.to_string(),
        })
    }
}
