use std::fs;
use std::path::Path;

use anyhow::format_err;
use anyhow::Result;
use log::error;

use super::dependency::DependencyManager;
use crate::core::Dependency;
use crate::core::DependencyLock;
use crate::core::Repository;
use crate::core::Spec;
use crate::core::SpecLock;

pub struct VendorManager<'a> {
    cache: &'a Path,
    spec: &'a mut Spec,
    lock: &'a mut SpecLock,
}

impl<'a> VendorManager<'a> {
    pub fn new<P: AsRef<Path>>(cache: &'a P, spec: &'a mut Spec, lock: &'a mut SpecLock) -> Self {
        VendorManager {
            cache: cache.as_ref(),
            spec,
            lock,
        }
    }

    pub fn install(&mut self) -> Result<()> {
        self.execute(&Self::inner_install)
    }

    pub fn update(&mut self) -> Result<()> {
        self.execute(&Self::inner_update)
    }

    fn inner_install(&mut self, dependency: &Dependency) -> Result<DependencyLock> {
        let repository = Repository::new(self.cache, dependency);
        let dependency_lock = self.lock.find_dep(&dependency.url);
        let dependency_manager =
            DependencyManager::new(self.spec, dependency, dependency_lock, &repository);
        dependency_manager.install(&self.spec.vendor)
    }

    fn inner_update(&mut self, dependency: &Dependency) -> Result<DependencyLock> {
        let repository = Repository::new(self.cache, dependency);
        let dependency_manager = DependencyManager::new(self.spec, dependency, None, &repository);
        dependency_manager.update(&self.spec.vendor)
    }

    fn execute(
        &mut self,
        action: &dyn Fn(&mut Self, &Dependency) -> Result<DependencyLock>,
    ) -> Result<()> {
        recreate_vendor_path(&self.spec.vendor)?;
        for dependency in self.spec.deps.clone().iter_mut() {
            let result = action(self, dependency);
            self.update_lock(dependency, result)?;
        }
        Ok(())
    }

    fn update_lock(
        &mut self,
        dependency: &Dependency,
        result: Result<DependencyLock>,
    ) -> Result<()> {
        match result {
            Ok(updated_dependency_lock) => {
                self.lock.add(updated_dependency_lock);
                Ok(())
            }
            Err(err) => {
                error!("failed importing {}: {}", dependency.url, err);
                Err(err)
            }
        }
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
            .map_err(|err| format_err!("cannot reset vendor folder: {}", err))?
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
        })?
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
    use crate::core::tests;

    #[test]
    fn test_ensure_vendor_empty_root() {
        let root = &tests::tempdir();
        let vendor = root.path().join("vendor");

        match create_vendor_path(&vendor) {
            Ok(()) => {
                assert!(vendor.exists());
                assert!(vendor.is_dir());
            }
            Err(err) => {
                panic!("expected vendor to succeed, but failed with: {}", err);
            }
        }
    }

    #[test]
    fn test_ensure_vendor_err_vendor_is_file() {
        let root = &tests::tempdir();
        let vendor = root.path().join("vendor");
        tests::write_to(&vendor, "");

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
                )
            }
        }
    }
}
