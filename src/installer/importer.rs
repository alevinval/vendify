use std::path::PathBuf;

use anyhow::Result;
use log::debug;
use log::info;

use super::collector::Collector;
use super::selector::Selector;
use crate::deps::Dependency;
use crate::deps::LockedDependency;
use crate::repository::Repository;
use crate::spec::Spec;

pub struct Importer<'a> {
    dependency: &'a Dependency,
    dependency_lock: Option<&'a LockedDependency>,
    repository: &'a Repository,
    collector: Collector,
    to: PathBuf,
}

impl<'a> Importer<'a> {
    pub fn new(
        spec: &'a Spec,
        dependency: &'a Dependency,
        dependency_lock: Option<&'a LockedDependency>,
        repository: &'a Repository,
    ) -> Self {
        let mut combined_filters = spec.filters.clone();
        combined_filters.merge(&dependency.filters);
        Self {
            dependency,
            dependency_lock,
            repository,
            collector: Selector::from(combined_filters).into(),
            to: PathBuf::from(&spec.vendor),
        }
    }

    /// Install copies the files of the dependency into the vendor folder.
    /// It respects the dependency lock, when passed.
    pub fn install(&self) -> Result<LockedDependency> {
        let refname = self.get_locked_refname();

        info!("installing {}@{}", self.dependency.url, refname);
        self.repository.fetch(&self.dependency.refname)?;
        self.repository.checkout(refname)?;
        self.import()
    }

    /// Update fetches latest changes from the git remote, against the
    /// reference. Then it installs the dependency. This will ignore the
    /// lock file and generate a new lock with the updated reference.
    pub fn update(&self) -> Result<LockedDependency> {
        let refname = self.dependency.refname.as_str();

        info!("updating {}@{}", self.dependency.url, refname);
        self.repository.fetch(refname)?;
        self.repository.reset(refname)?;
        self.import()
    }

    fn import(&self) -> Result<LockedDependency> {
        self.copy_files()?;
        let locked = self.get_locked_dependency()?;
        info!("\t🔒 {}", locked.refname);
        Ok(locked)
    }

    fn copy_files(&self) -> Result<()> {
        for collected in self.collector.collect(&self.repository.path()) {
            debug!(
                "\t.../{} -> {}",
                collected.src_rel.display(),
                self.to.join(&collected.src_rel).display()
            );
            collected.copy(&self.to)?;
        }
        Ok(())
    }

    fn get_locked_refname(&self) -> &str {
        match self.dependency_lock {
            Some(it) => &it.refname,
            None => &self.dependency.refname,
        }
    }

    fn get_locked_dependency(&self) -> Result<LockedDependency> {
        let refname = self.repository.get_current_refname()?;
        Ok(self.dependency.to_locked_dependency(refname))
    }
}
