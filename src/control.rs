use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;

use log::error;
use log::info;
use log::warn;

use super::deps::Dependency;
use super::installer::Installer;
use super::spec::Spec;
use super::spec_lock::SpecLock;
use crate::cache::Cache;
use crate::preset::Preset;

pub struct Controller {
    preset: Arc<Preset>,
}

impl Controller {
    pub fn new(preset: Preset) -> Self {
        Self {
            preset: Arc::new(preset),
        }
    }

    pub fn init(&self) {
        info!("initializing vendor in current directory");

        let spec_path: &PathBuf = &self.preset.spec().into();
        if spec_path.exists() {
            warn!("{} already exists", spec_path.display());
            return;
        }

        let mut spec = Spec::with_preset(self.preset.clone());
        if let Err(err) = spec.save() {
            error!("{err}");
            return;
        }

        info!("{} has been created", spec_path.display());
    }

    pub fn add(
        &self,
        url: &str,
        refname: &str,
        extensions: Option<Vec<String>>,
        targets: Option<Vec<String>>,
        ignores: Option<Vec<String>>,
    ) {
        let mut spec = match Spec::load_from(self.preset.clone()) {
            Ok(spec) => spec,
            Err(err) => {
                error!("{err}");
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
        spec.add_dependency(dep);

        match spec.save() {
            Ok(_) => {
                info!("added dependency {url}@{refname}");
            }
            Err(err) => {
                error!("cannot add dependency: {err}");
            }
        }
    }

    pub fn install(&self) {
        let spec = match Spec::load_from(self.preset.clone()) {
            Ok(value) => Arc::new(RwLock::new(value)),
            Err(err) => {
                error!("{err}");
                return;
            }
        };

        let lock = match SpecLock::load_from(self.preset.clone()) {
            Ok(value) => Arc::new(RwLock::new(value)),
            Err(_) => Arc::new(RwLock::new(SpecLock::with_preset(self.preset.clone()))),
        };

        let cache = Cache::new(&self.preset);
        let installer = Installer::new(cache, spec.clone(), lock.clone());
        if let Err(err) = installer.install() {
            error!("install failed: {err}");
            return;
        };

        if let Err(err) = lock.write().unwrap().save() {
            error!("install failed: {err}");
            return;
        }

        if let Err(err) = spec.write().unwrap().save() {
            error!("install failed: {err}");
            return;
        }

        info!("install success ✅");
    }

    pub fn update(&self) {
        let spec = match Spec::load_from(self.preset.clone()) {
            Ok(value) => Arc::new(RwLock::new(value)),
            Err(err) => {
                error!("{}", err);
                return;
            }
        };

        let lock = match SpecLock::load_from(self.preset.clone()) {
            Ok(value) => Arc::new(RwLock::new(value)),
            Err(_) => Arc::new(RwLock::new(SpecLock::with_preset(self.preset.clone()))),
        };

        let cache = Cache::new(&self.preset);
        let installer = Installer::new(cache, spec.clone(), lock.clone());
        if let Err(err) = installer.update() {
            error!("update failed: {err}");
            return;
        };

        if let Err(err) = lock.write().unwrap().save() {
            error!("update failed: {err}");
            return;
        }

        if let Err(err) = spec.write().unwrap().save() {
            error!("update failed: {err}");
            return;
        }

        info!("update success ✅");
    }
}
