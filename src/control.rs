use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;

use anyhow::Result;
use log::error;
use log::info;
use log::warn;

use super::deps::Dependency;
use super::installer::Installer;
use super::spec::Spec;
use super::spec_lock::SpecLock;
use crate::cache::Cache;
use crate::preset::Preset;

type SharedSpec = Arc<RwLock<Spec>>;
type SharedSpecLock = Arc<RwLock<SpecLock>>;

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

    pub fn install(&self) -> Result<()> {
        let (spec, spec_lock) = self.load_both()?;
        let cache = Cache::new(&self.preset);
        let _cache_lock = cache.lock();
        let installer = Installer::new(cache, spec.clone(), spec_lock.clone());

        let run_install = || -> Result<()> {
            installer.install()?;
            spec_lock.write().unwrap().save()?;
            spec.write().unwrap().save()?;
            Ok(())
        };

        if let Err(err) = run_install() {
            error!("install failed: {err}");
            return Err(err);
        };

        info!("install success ✅");
        Ok(())
    }

    pub fn update(&self) -> Result<()> {
        let (spec, spec_lock) = self.load_both()?;
        let cache = Cache::new(&self.preset);
        let _cache_lock = cache.lock();
        let installer = Installer::new(cache, spec.clone(), spec_lock.clone());

        let run_update = || -> Result<()> {
            installer.update()?;
            spec_lock.write().unwrap().save()?;
            spec.write().unwrap().save()?;
            Ok(())
        };

        if let Err(err) = run_update() {
            error!("update failed: {err}");
            return Err(err);
        };

        info!("update success ✅");
        Ok(())
    }

    fn load_both(&self) -> Result<(SharedSpec, SharedSpecLock)> {
        let spec = Self::wrap(match Spec::load_from(self.preset.clone()) {
            Ok(value) => value,
            Err(err) => {
                error!("{err}");
                return Err(err);
            }
        });

        let spec_lock = Self::wrap(match SpecLock::load_from(self.preset.clone()) {
            Ok(value) => value,
            Err(_) => SpecLock::with_preset(self.preset.clone()),
        });

        Ok((spec, spec_lock))
    }

    fn wrap<T>(input: T) -> Arc<RwLock<T>> {
        Arc::new(RwLock::new(input))
    }
}
