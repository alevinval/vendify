use std::path::Path;
use std::sync::Arc;
use std::sync::RwLock;

use log::error;
use log::info;
use log::warn;

use super::cache::CacheManager;
use super::dependency::Dependency;
use super::installer::Installer;
use super::loadable_config::LoadableConfig;
use super::spec::Spec;
use super::spec_lock::SpecLock;
use super::VENDOR_LOCK_YML;
use super::VENDOR_YML;

pub struct Controller {
    cache: CacheManager,
}

impl Controller {
    pub fn new() -> Self {
        ControllerBuilder::new().build()
    }

    pub fn init(self) {
        info!("initializing vendor in current directory");

        if Path::new(VENDOR_YML).exists() {
            warn!("{} already exists", VENDOR_YML);
            return;
        }

        let mut config = Spec::new();
        if let Err(err) = config.save_to(VENDOR_YML) {
            error!("failed initializing: {}", err);
            return;
        }

        info!("{} has been created", VENDOR_YML);
    }

    pub fn add(
        self,
        url: &str,
        refname: &str,
        extensions: Option<Vec<String>>,
        targets: Option<Vec<String>>,
        ignores: Option<Vec<String>>,
    ) {
        let mut spec = match Spec::load_from(VENDOR_YML) {
            Ok(config) => config,
            Err(err) => {
                error!("{}", err);
                return;
            }
        };

        let mut dep = Dependency::new(url, refname);
        if let Some(extensions) = extensions {
            dep.filters.add_extensions(&extensions);
        }
        if let Some(targets) = targets {
            dep.filters.add_targets(&targets);
        }
        if let Some(ignores) = ignores {
            dep.filters.add_ignores(&ignores);
        }
        spec.add(dep);

        match spec.save_to(VENDOR_YML) {
            Ok(_) => {
                info!("added dependency {}@{}", url, refname);
            }
            Err(err) => {
                error!("add failed: {}", err)
            }
        }
    }

    pub fn install(self) {
        let spec = match Spec::load_from(VENDOR_YML) {
            Ok(value) => Arc::new(RwLock::new(value)),
            Err(err) => {
                error!("{}", err);
                return;
            }
        };

        let lock = match SpecLock::load_from(VENDOR_LOCK_YML) {
            Ok(value) => Arc::new(RwLock::new(value)),
            Err(_) => Arc::new(RwLock::new(SpecLock::new())),
        };

        let manager = Installer::new(Arc::clone(&spec), Arc::clone(&lock));
        if let Err(err) = manager.install() {
            error!("install failed: {}", err);
            return;
        };

        if let Err(err) = lock.write().unwrap().save_to(VENDOR_LOCK_YML) {
            error!("install failed: {}", err);
            return;
        }

        if let Err(err) = spec.write().unwrap().save_to(VENDOR_YML) {
            error!("install failed: {}", err);
            return;
        }

        info!("install success ✅")
    }

    pub fn update(self) {
        let spec = match Spec::load_from(VENDOR_YML) {
            Ok(value) => Arc::new(RwLock::new(value)),
            Err(err) => {
                error!("{}", err);
                return;
            }
        };

        let lock = match SpecLock::load_from(VENDOR_LOCK_YML) {
            Ok(value) => Arc::new(RwLock::new(value)),
            Err(_) => Arc::new(RwLock::new(SpecLock::new())),
        };

        let manager = Installer::new(Arc::clone(&spec), Arc::clone(&lock));
        if let Err(err) = manager.update() {
            error!("update failed: {}", err);
            return;
        };

        if let Err(err) = lock.write().unwrap().save_to(VENDOR_LOCK_YML) {
            error!("update failed: {}", err);
            return;
        }

        if let Err(err) = spec.write().unwrap().save_to(VENDOR_YML) {
            error!("update failed: {}", err);
            return;
        }

        info!("update success ✅")
    }
}

pub struct ControllerBuilder {
    cache: CacheManager,
}

impl ControllerBuilder {
    pub fn new() -> Self {
        ControllerBuilder {
            cache: CacheManager::new(),
        }
    }

    pub fn cache(mut self, cache: CacheManager) -> Self {
        self.cache = cache;
        self
    }

    pub fn build(self) -> Controller {
        Controller { cache: self.cache }
    }
}
