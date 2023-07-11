use std::fs;
use std::path::Path;
use std::thread;

use anyhow::format_err;
use anyhow::Result;

use self::importer::Importer;
use crate::cache::Cache;
use crate::deps::Dependency;
use crate::deps::LockedDependency;
use crate::spec::Spec;
use crate::spec_lock::SpecLock;

mod collector;
mod importer;
mod selector;

pub struct Installer<'spec> {
    cache: Cache,
    spec: &'spec Spec,
    spec_lock: SpecLock,
}

impl<'spec> Installer<'spec> {
    pub fn new(cache: Cache, spec: &'spec Spec, spec_lock: SpecLock) -> Self {
        Self {
            cache,
            spec,
            spec_lock,
        }
    }

    pub fn install(self) -> Result<SpecLock> {
        self.execute(Self::inner_install)
    }

    pub fn update(self) -> Result<SpecLock> {
        self.execute(Self::inner_update)
    }

    fn execute<F>(mut self, callback: F) -> Result<SpecLock>
    where
        F: (Fn(&Installer<'spec>, &Dependency) -> Result<LockedDependency>) + Sync + Send,
    {
        self.cache.initialize()?;
        recreate_vendor_path(&self.spec.vendor)?;

        let updated_locks: Vec<_> = thread::scope(|s| {
            self.spec
                .deps
                .iter()
                .map(|dep| s.spawn(|| callback(&self, dep)))
                .filter_map(|handle| handle.join().ok())
                .flatten()
                .collect()
        });

        for lock in updated_locks {
            self.spec_lock.add_locked_dependency(lock);
        }

        Ok(self.spec_lock)
    }

    fn inner_install(&self, dependency: &Dependency) -> Result<LockedDependency> {
        let _repository_lock = self.cache.lock_repository(dependency)?;
        let repository = self.cache.get_repository(dependency)?;
        let dependency_lock = self.spec_lock.get_locked_dependency(&dependency.url);
        let importer = Importer::new(self.spec, dependency, dependency_lock, &repository);

        importer.install()
    }

    fn inner_update(&self, dependency: &Dependency) -> Result<LockedDependency> {
        let _repository_lock = self.cache.lock_repository(dependency)?;
        let repository = self.cache.get_repository(dependency)?;
        let importer = Importer::new(self.spec, dependency, None, &repository);

        importer.update()
    }
}

fn recreate_vendor_path<P: AsRef<Path>>(path: P) -> Result<()> {
    delete_vendor_path(&path)?;
    create_vendor_path(&path)
}

fn delete_vendor_path<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    if path.exists() {
        fs::remove_dir_all(path)
            .map_err(|err| format_err!("cannot reset vendor folder: {}", err))?;
    }
    Ok(())
}

fn create_vendor_path<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    if !path.exists() {
        fs::create_dir_all(path).map_err(|err| {
            format_err!(
                "cannot create vendor folder '{name}': {err}",
                name = path.display(),
                err = err
            )
        })?;
    }
    if !path.is_dir() {
        return Err(format_err!(
            "vendor path '{}' already exists, and it's not a directory",
            path.display()
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_utils::tempdir;
    use crate::test_utils::write_to;

    #[test]
    fn test_ensure_vendor_empty_root() {
        let root = tempdir();
        let vendor = root.path().join("vendor");

        match create_vendor_path(&vendor) {
            Ok(()) => {
                assert!(vendor.exists());
                assert!(vendor.is_dir());
            }
            Err(err) => {
                panic!("expected vendor to succeed, but failed with: {err}");
            }
        }
    }

    #[test]
    fn test_ensure_vendor_err_vendor_is_file() {
        let root = &tests::tempdir();
        let vendor = root.path().join("vendor");
        write_to(&vendor, "");

        match create_vendor_path(&vendor) {
            Ok(()) => {
                panic!("expected to fail, but succeeded with: {}", vendor.display());
            }
            Err(err) => {
                assert_eq!(
                    format!(
                        "vendor path '{}' already exists, and it's not a directory",
                        vendor.display()
                    ),
                    err.to_string()
                );
            }
        }
    }
}
