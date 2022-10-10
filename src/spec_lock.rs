use std::sync::Arc;

use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

use crate::deps::LockedDependency;
use crate::preset::Preset;
use crate::yaml;
use crate::VERSION;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SpecLock {
    /// Version that was used to generate the config
    pub version: String,

    /// List of locked dependencies
    pub deps: Vec<LockedDependency>,

    #[serde(skip)]
    preset: Arc<Preset>,
}

impl SpecLock {
    pub fn with_preset(preset: Arc<Preset>) -> Self {
        let mut lock = Self {
            version: VERSION.to_owned(),
            deps: Vec::new(),
            preset: preset.clone(),
        };
        lock.apply_preset(preset);
        lock
    }

    pub fn load_from(preset: Arc<Preset>) -> Result<Self> {
        let mut lock: SpecLock = yaml::load(preset.spec_lock())?;
        lock.apply_preset(preset);
        Ok(lock)
    }

    pub fn save(&mut self) -> Result<()> {
        self.lint();
        yaml::save(self, self.preset.spec_lock())
    }

    pub fn apply_preset(&mut self, preset: Arc<Preset>) {
        if self.version < VERSION.into() {
            self.version = VERSION.into();
        }
        self.preset = preset;
    }

    pub fn add_locked_dependency(&mut self, dep: LockedDependency) {
        match self.get_mut_locked_dependency(&dep.url) {
            Some(found) => {
                found.refname = dep.refname;
            }
            None => {
                self.deps.push(dep);
            }
        }
    }

    pub fn get_locked_dependency(&self, url: &str) -> Option<&LockedDependency> {
        self.deps.iter().find(|l| l.url.eq_ignore_ascii_case(url))
    }

    fn get_mut_locked_dependency(&mut self, url: &str) -> Option<&mut LockedDependency> {
        self.deps
            .iter_mut()
            .find(|l| l.url.eq_ignore_ascii_case(url))
    }

    fn lint(&mut self) {
        self.deps.sort_by(|a, b| a.url.cmp(&b.url));
        self.deps
            .dedup_by(|a, b| a.url.eq_ignore_ascii_case(&b.url));
    }

    #[cfg(test)]
    pub fn new() -> Self {
        Self::with_preset(Arc::new(Preset::default()))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_utils::build_preset;
    use crate::test_utils::TestContext;

    #[test]
    fn test_spec_lock_new() {
        let sut = SpecLock::new();

        assert_eq!(VERSION, sut.version, "should have the crate version");
        assert_eq!(0, sut.deps.len(), "should have no deps");
        assert_eq!(
            &Preset::default(),
            sut.preset.as_ref(),
            "should use default preset"
        );
    }

    #[test]
    fn test_spec_lock_from_preset() {
        let preset = build_preset();
        let sut = SpecLock::with_preset(preset.clone());

        assert_eq!(VERSION, sut.version, "should have the crate version");
        assert_eq!(0, sut.deps.len(), "should have no deps");
        assert_eq!(*preset, *sut.preset, "should use the provided preset");
    }

    #[test]
    fn test_spec_lock_add_dependency() {
        let mut sut = SpecLock::new();
        let dep = LockedDependency::new("some-url", "some-refname");

        sut.add_locked_dependency(dep.clone());

        assert_eq!(1, sut.deps.len());
        assert_eq!(dep, sut.deps[0]);
    }

    #[test]
    fn test_spec_lock_apply_preset_updates_version() -> Result<()> {
        let preset = &TestContext::new();
        let mut sut = SpecLock::with_preset(preset.into());
        sut.version = "0.0.0".into();

        sut.save()?;

        let actual = SpecLock::load_from(preset.into())?;
        assert_eq!(VERSION, actual.version);
        Ok(())
    }

    #[test]
    fn test_spec_lock_with_preset_then_save_then_load() -> Result<()> {
        let dep = LockedDependency::new("some url", "some ref");
        let context = &TestContext::new();
        let mut expected = SpecLock::with_preset(context.into());
        expected.add_locked_dependency(dep);

        expected.save()?;

        let actual = SpecLock::load_from(context.into()).expect("loaded file");
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_spec_lock_cannot_load_from_non_existent_file() {
        let context = &TestContext::new();
        let actual = SpecLock::load_from(context.into());
        assert!(actual.is_err(), "there should be an error");
    }
}
