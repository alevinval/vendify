use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use std::thread::ScopedJoinHandle;

use anyhow::format_err;
use anyhow::Result;
use log::error;

use crate::cache::CacheManager;
use crate::dependency::Dependency;
use crate::dependency::DependencyLock;
use crate::importer::Importer;
use crate::spec::Spec;
use crate::spec_lock::SpecLock;

type ActionFn = dyn Fn(&Installer, &Dependency) -> Result<DependencyLock> + Sync + Send;

pub struct Installer {
    cache: CacheManager,
    spec: Arc<RwLock<Spec>>,
    spec_lock: Arc<RwLock<SpecLock>>,
}

impl Installer {
    pub fn new(spec: Arc<RwLock<Spec>>, lock: Arc<RwLock<SpecLock>>) -> Self {
        Installer {
            cache: CacheManager::new(),
            spec,
            spec_lock: lock,
        }
    }

    pub fn install(self) -> Result<()> {
        self.execute(Arc::new(inner_install))
    }

    pub fn update(self) -> Result<()> {
        self.execute(Arc::new(inner_update))
    }

    fn execute(self, action: Arc<ActionFn>) -> Result<()> {
        self.cache.ensure()?;
        recreate_vendor_path(&self.spec.read().unwrap().vendor)?;

        let deps = &self.spec.read().unwrap().deps;

        thread::scope(|s| {
            let mut handles: Vec<ScopedJoinHandle<Result<DependencyLock>>> = vec![];

            for dep in deps.iter() {
                handles.push(s.spawn(|| action(&self, dep)));
            }

            for handle in handles.into_iter() {
                if let Ok(result) = handle.join() {
                    self.update_lock(result)
                }
            }
        });

        Ok(())
    }

    fn update_lock(&self, result: Result<DependencyLock>) {
        match result {
            Ok(updated_dependency_lock) => {
                self.spec_lock.write().unwrap().add(updated_dependency_lock);
            }
            Err(err) => {
                error!("failed importing: {}", err);
            }
        }
    }
}

fn inner_install(installer: &Installer, dependency: &Dependency) -> Result<DependencyLock> {
    let repository = installer.cache.get_repository(dependency)?;
    let spec = installer.spec.read().unwrap();
    let spec_lock = installer.spec_lock.read().unwrap();
    let dependency_lock = spec_lock.find_dep(&dependency.url);
    let importer = Importer::new(&spec, dependency, dependency_lock, &repository);

    importer.install()
}

fn inner_update(installer: &Installer, dependency: &Dependency) -> Result<DependencyLock> {
    let repository = installer.cache.get_repository(dependency)?;
    let spec = installer.spec.read().unwrap();
    let importer = Importer::new(&spec, dependency, None, &repository);

    importer.update()
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
                panic!("expected vendor to succeed, but failed with: {}", err);
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
                )
            }
        }
    }
}
