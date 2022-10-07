use std::fmt::Debug;

use serde::Deserialize;
use serde::Serialize;

use super::filters::Filters;
use crate::core::Dependency;
use crate::core::LoadableConfig;
use crate::VERSION;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Spec {
    /// Version that was used to generate the spec
    pub version: String,

    /// Vendor directory path
    #[serde(default = "default_vendor")]
    pub vendor: String,

    #[serde(flatten)]
    pub filters: Filters,

    /// List of dependencies
    pub deps: Vec<Dependency>,
}

impl Spec {
    pub fn new() -> Self {
        Spec {
            version: VERSION.to_string(),
            vendor: default_vendor(),
            filters: Filters::new(),
            deps: vec![],
        }
    }

    pub fn add(&mut self, dep: Dependency) {
        match self.find_dep(&dep) {
            Some(existing) => existing.update_from(&dep),
            None => self.deps.push(dep),
        }
    }

    fn find_dep(&mut self, dep: &Dependency) -> Option<&mut Dependency> {
        self.deps
            .iter_mut()
            .find(|d| d.url.eq_ignore_ascii_case(&dep.url))
    }
}

impl LoadableConfig<Spec> for Spec {
    fn lint(&mut self) {
        self.deps.sort_by(|a, b| a.url.cmp(&b.url));
        self.deps
            .dedup_by(|a, b| a.url.eq_ignore_ascii_case(&b.url));
    }
}

fn default_vendor() -> String {
    "vendor".to_string()
}

#[cfg(test)]
mod tests {

    use std::io::Write;

    use anyhow::Result;

    use super::*;
    use crate::core::tests::test_util::tempfile;

    #[test]
    fn test_new_default_instance() {
        let sut = Spec::new();

        assert_eq!(
            VERSION, sut.version,
            "default instance version should be crate version"
        );
        assert_eq!(0, sut.deps.len(), "default instance should have no deps");
    }

    #[test]
    fn test_add_dependency() {
        let mut sut = Spec::new();
        let dep = Dependency::new("some url", "some ref");

        sut.add(dep.clone());

        assert_eq!(1, sut.deps.len());
        assert_eq!(dep, sut.deps.first().unwrap().to_owned());
    }

    #[test]
    fn test_initialize_save_then_load() -> Result<()> {
        let tmp = tempfile();
        let dep = Dependency::new("some url", "some ref");
        let mut sut = Spec::new();
        sut.add(dep);

        sut.save_to(&tmp)?;
        let actual = Spec::load_from(&tmp)?;

        assert_eq!(sut, actual);

        Ok(())
    }

    #[test]
    fn test_cannot_load_invalid_file() -> Result<()> {
        let mut out = tempfile();
        out.write(b"bf")?;
        out.flush()?;

        let actual = Spec::load_from(out);
        assert!(actual.is_err(), "there should be an error");

        Ok(())
    }
}
