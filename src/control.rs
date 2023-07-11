use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use log::error;
use log::info;
use log::warn;

use super::deps::Dependency;
use super::installer::Installer;
use super::spec::Spec;
use super::spec_lock::SpecLock;
use crate::cache::Cache;
use crate::filters::FilterKind;
use crate::preset::Preset;

pub struct Controller {
    preset: Preset,
}

impl Controller {
    pub fn new(preset: Preset) -> Self {
        Self { preset }
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
            dep.filters.add(FilterKind::Extension(extensions));
        }
        if let Some(targets) = targets {
            dep.filters.add(FilterKind::Target(targets));
        }
        if let Some(ignores) = ignores {
            dep.filters.add(FilterKind::Ignore(ignores));
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
        let (mut spec, spec_lock) = self.load_both()?;
        let cache = Cache::new(&self.preset);
        let _cache_lock = cache.lock();
        let installer = Installer::new(cache, &spec, spec_lock);

        if let Err(err) = {
            let mut spec_lock = installer.install()?;
            spec_lock.save()?;
            spec.save()?;
            Ok(())
        } {
            error!("install failed: {err}");
            return Err(err);
        };

        info!("install success ✅");
        Ok(())
    }

    pub fn update(&self) -> Result<()> {
        let (mut spec, spec_lock) = self.load_both()?;
        let cache = Cache::new(&self.preset);
        let _cache_lock = cache.lock();
        let installer = Installer::new(cache, &spec, spec_lock);

        if let Err(err) = {
            let mut spec_lock = installer.update()?;
            spec_lock.save()?;
            spec.save()?;
            Ok(())
        } {
            error!("update failed: {err}");
            return Err(err);
        };

        info!("update success ✅");
        Ok(())
    }

    pub fn clear_cache(&self) -> Result<()> {
        Cache::new(&self.preset).clear()
    }

    fn load_both(&self) -> Result<(Spec, SpecLock)> {
        let spec = match Spec::load_from(self.preset.clone()) {
            Ok(value) => value,
            Err(err) => {
                error!("{err}");
                return Err(err);
            }
        };

        let spec_lock = match SpecLock::load_from(self.preset.clone()) {
            Ok(value) => value,
            Err(_) => SpecLock::with_preset(self.preset.clone()),
        };

        Ok((spec, spec_lock))
    }
}
