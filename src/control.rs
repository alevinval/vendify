use std::path::PathBuf;

use anyhow::Result;

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
        log::info!("initializing vendor in current directory");

        let spec_path: &PathBuf = &self.preset.spec().into();
        if spec_path.exists() {
            log::warn!("{} already exists", spec_path.display());
            return;
        }

        let mut spec = Spec::with_preset(&self.preset);
        if let Err(err) = spec.save() {
            log::error!("{err}");
            return;
        }

        log::info!("{} has been created", spec_path.display());
    }

    pub fn add(
        &self,
        url: &str,
        refname: &str,
        extensions: Option<Vec<String>>,
        targets: Option<Vec<String>>,
        ignores: Option<Vec<String>>,
    ) {
        let mut spec = match Spec::load_from(&self.preset) {
            Ok(spec) => spec,
            Err(err) => {
                log::error!("{err}");
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
                log::info!("added dependency {url}@{refname}");
            }
            Err(err) => {
                log::error!("cannot add dependency: {err}");
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
            log::error!("install failed: {err}");
            return Err(err);
        };

        log::info!("install success ✅");
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
            log::error!("update failed: {err}");
            return Err(err);
        };

        log::info!("update success ✅");
        Ok(())
    }

    pub fn clear_cache(&self) -> Result<()> {
        Cache::new(&self.preset).clear()
    }

    fn load_both(&self) -> Result<(Spec, SpecLock)> {
        let spec = match Spec::load_from(&self.preset) {
            Ok(value) => value,
            Err(err) => {
                log::error!("{err}");
                return Err(err);
            }
        };

        let spec_lock = match SpecLock::load_from(&self.preset) {
            Ok(value) => value,
            Err(_) => SpecLock::with_preset(&self.preset),
        };

        Ok((spec, spec_lock))
    }
}
