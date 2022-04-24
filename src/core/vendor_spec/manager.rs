use super::vendor::ensure_vendor;
use super::vendor::reset_vendor;
use super::VendorSpec;
use crate::core::dependency::DependencyManager;
use crate::core::VendorLock;
use anyhow::Result;
use log::error;
use std::path::Path;

pub struct VendorManager<'a> {
    cache: &'a Path,
    spec: &'a mut VendorSpec,
    lock: &'a mut VendorLock,
}

impl<'a> VendorManager<'a> {
    pub fn new<P: AsRef<Path>>(
        cache: &'a P,
        spec: &'a mut VendorSpec,
        lock: &'a mut VendorLock,
    ) -> Self {
        VendorManager {
            cache: cache.as_ref(),
            spec,
            lock,
        }
    }

    pub fn install(&mut self) -> Result<()> {
        reset_vendor(&self.spec.vendor)?;
        let vendor = ensure_vendor(&self.spec.vendor)?;
        for dep in &self.spec.deps {
            let locked_dep = self.lock.find_dep(&dep.url);
            let manager = DependencyManager::new(self.cache, dep, locked_dep);
            match manager.install(&vendor) {
                Ok(locked_dep) => {
                    self.lock.add(locked_dep);
                }
                Err(err) => {
                    error!("failed importing {}: {}", &dep.url, err);
                    return Err(err);
                }
            }
        }
        Ok(())
    }

    pub fn update(&mut self) -> Result<()> {
        let vendor = ensure_vendor(&self.spec.vendor)?;
        for dep in &self.spec.deps {
            let manager = DependencyManager::new(self.cache, dep, None);
            match manager.update(&vendor) {
                Ok(locked_dep) => {
                    self.lock.add(locked_dep);
                }
                Err(err) => {
                    error!("failed importing {}: {}", &dep.url, err);
                    return Err(err);
                }
            }
        }
        Ok(())
    }
}
