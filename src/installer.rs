use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use std::thread::ScopedJoinHandle;

use anyhow::format_err;
use anyhow::Result;
use log::error;

use self::importer::Importer;
use crate::cache::Cache;
use crate::deps::Dependency;
use crate::deps::LockedDependency;
use crate::spec::Spec;
use crate::spec_lock::SpecLock;

mod collector;
mod importer;
mod selector;

type ActionFn = fn(&Installer, &Dependency) -> Result<LockedDependency>;

pub struct Installer {
    cache: Cache,
    spec: Arc<RwLock<Spec>>,
    spec_lock: Arc<RwLock<SpecLock>>,
}

impl Installer {
    pub fn new(cache: Cache, spec: Arc<RwLock<Spec>>, spec_lock: Arc<RwLock<SpecLock>>) -> Self {
        Self {
            cache,
            spec,
            spec_lock,
        }
    }

    pub fn install(&self) -> Result<()> {
        self.execute(Self::inner_install)
    }

    pub fn update(&self) -> Result<()> {
        self.execute(Self::inner_update)
    }

    fn execute(&self, action: ActionFn) -> Result<()> {
        self.cache.ensure()?;
        recreate_vendor_path(&self.spec.read().unwrap().vendor)?;

        let deps = &self.spec.read().unwrap().deps;
        thread::scope(|s| {
            let mut handles: Vec<ScopedJoinHandle<Result<LockedDependency>>> = vec![];

            for dep in deps.iter() {
                handles.push(s.spawn(|| action(self, dep)));
            }

            for handle in handles {
                if let Ok(result) = handle.join() {
                    self.update_lock(result);
                }
            }
        });

        Ok(())
    }

    fn update_lock(&self, result: Result<LockedDependency>) {
        match result {
            Ok(dep) => {
                self.spec_lock.write().unwrap().add_locked_dependency(dep);
            }
            Err(err) => {
                error!("failed importing: {}", err);
            }
        }
    }

    fn inner_install(&self, dependency: &Dependency) -> Result<LockedDependency> {
        let repository = self.cache.get_repository(dependency)?;
        let spec = self.spec.read().unwrap();
        let spec_lock = self.spec_lock.read().unwrap();
        let dependency_lock = spec_lock.get_locked_dependency(&dependency.url);
        let importer = Importer::new(&spec, dependency, dependency_lock, &repository);

        importer.install()
    }

    fn inner_update(&self, dependency: &Dependency) -> Result<LockedDependency> {
        let repository = self.cache.get_repository(dependency)?;
        let spec = self.spec.read().unwrap();
        let importer = Importer::new(&spec, dependency, None, &repository);

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
