use std::fmt::Debug;

use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

use crate::deps::Dependency;
use crate::filters::Filters;
use crate::preset::Preset;
use crate::yaml;
use crate::VERSION;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Spec {
    /// Version that was used to generate the spec.
    pub version: String,

    // Name of the preset used to generate this spec file.
    #[serde(default, rename = "preset")]
    preset_name: String,

    /// Vendor directory path.
    pub vendor: String,

    #[serde(flatten)]
    pub filters: Filters,

    /// List of dependencies.
    pub deps: Vec<Dependency>,

    #[serde(skip)]
    preset: Preset,
}

impl Spec {
    pub fn with_preset(preset: &Preset) -> Self {
        let mut spec = Self {
            version: VERSION.to_string(),
            vendor: String::new(),
            filters: Filters::new(),
            deps: vec![],
            preset_name: preset.name().to_string(),
            preset: preset.clone(),
        };
        spec.apply_preset();
        spec
    }

    pub fn add_dependency(&mut self, mut dep: Dependency) {
        dep.apply_preset(&self.preset);
        if let Some(existing) = self.get_mut_dependency(&dep) {
            existing.update_from(&dep);
        } else {
            self.deps.push(dep);
        }
    }

    pub fn load_from(preset: &Preset) -> Result<Self> {
        let mut spec: Self = yaml::load(preset.spec())?;
        spec.preset = preset.clone();
        spec.apply_preset();
        Ok(spec)
    }

    pub fn save(&mut self) -> Result<()> {
        self.lint();
        yaml::save(self, self.preset.spec())
    }

    fn get_mut_dependency(&mut self, dep: &Dependency) -> Option<&mut Dependency> {
        self.deps
            .iter_mut()
            .find(|d| d.url.eq_ignore_ascii_case(&dep.url))
    }

    fn apply_preset(&mut self) {
        let crate_version = VERSION.to_string();
        if self.version < crate_version {
            self.version = crate_version;
        }
        self.vendor = self.preset.vendor().to_string();
        if self.preset.force_filters() {
            self.filters.clear();
        }
        self.filters.merge(&self.preset.global_filters());
        self.deps.iter_mut().for_each(|dep| {
            dep.apply_preset(&self.preset);
        });
        self.preset_name = self.preset.name().to_string();
    }

    fn lint(&mut self) {
        self.deps.sort_by(|a, b| a.url.cmp(&b.url));
        self.deps
            .dedup_by(|a, b| a.url.eq_ignore_ascii_case(&b.url));
    }

    #[cfg(test)]
    pub fn new() -> Self {
        Self::with_preset(&Preset::default())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_utils::build_preset;
    use crate::test_utils::TestContext;

    #[test]
    fn test_spec_new() {
        let sut = Spec::new();

        assert_eq!(VERSION, sut.version, "should have the crate version");
        assert_eq!("vendor", sut.vendor, "should have default vendor folder");
        assert_eq!(Filters::new(), sut.filters, "should have empty filters");
        assert_eq!(0, sut.deps.len(), "should have no deps");
        assert_eq!("default", sut.preset_name);
        assert_eq!(Preset::default(), sut.preset, "should use default preset");
    }

    #[test]
    fn test_spec_with_preset() {
        let preset = build_preset();
        let dep = Dependency::new("some-url", "some-refname");
        let mut sut = Spec::with_preset(&preset);
        sut.add_dependency(dep.clone());

        assert_eq!(VERSION, sut.version, "should have the crate version");
        assert_eq!(
            ".test-vendor", sut.vendor,
            "should have default vendor folder"
        );
        assert_eq!(
            &preset.global_filters(),
            &sut.filters,
            "should have expected filters"
        );
        assert_eq!(1, sut.deps.len(), "should have added dep");
        assert_eq!(
            preset.dependency_filters(&dep),
            sut.deps[0].filters,
            "should have dependency filters applied"
        );
        assert_eq!("test-preset", sut.preset_name);
        assert_eq!(preset, sut.preset, "should use the provided preset");
    }

    #[test]
    fn test_spec_add_dependency() {
        let preset = build_preset();
        let mut sut = Spec::with_preset(&preset);
        let mut dep = Dependency::new("some url", "some ref");

        sut.add_dependency(dep.clone());

        assert_eq!(1, sut.deps.len());
        assert_ne!(&dep, &sut.deps[0]);
        assert_eq!(dep.apply_preset(&preset), &sut.deps[0]);
    }

    #[test]
    fn test_spec_apply_preset_updates_version() -> Result<()> {
        let ctx = TestContext::new();
        let mut sut = Spec::with_preset(&ctx.preset);
        sut.version = "0.0.0".into();

        sut.save()?;

        let actual = Spec::load_from(&ctx.preset)?;
        assert_eq!(VERSION, actual.version);
        Ok(())
    }

    #[test]
    fn test_spec_with_preset_then_save_then_load() -> Result<()> {
        let ctx = TestContext::new();
        let dep = Dependency::new("some url", "some ref");
        let mut expected = Spec::with_preset(&ctx.preset);
        expected.add_dependency(dep);

        expected.save()?;

        let actual = Spec::load_from(&ctx.preset)?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_spec_cannot_load_from_non_existent_file() {
        let ctx = TestContext::new();
        let actual = Spec::load_from(&ctx.preset);
        assert!(actual.is_err(), "there should be an error");
    }
}
