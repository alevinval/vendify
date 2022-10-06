use serde::Deserialize;
use serde::Serialize;

use super::dependency::DependencyLock;
use super::LoadableConfig;
use crate::VERSION;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SpecLock {
    /// Version that was used to generate the config
    pub version: String,

    /// List of locked dependencies
    pub deps: Vec<DependencyLock>,
}

impl SpecLock {
    pub fn new() -> Self {
        SpecLock {
            version: VERSION.to_owned(),
            deps: Vec::new(),
        }
    }

    pub fn add(&mut self, dep: DependencyLock) {
        match self.find_dep_mut(&dep.url) {
            Some(found) => {
                found.refname = dep.refname.clone();
            }
            None => {
                self.deps.push(dep);
            }
        }
    }

    pub fn find_dep(&self, url: &str) -> Option<&DependencyLock> {
        self.deps.iter().find(|l| l.url.eq_ignore_ascii_case(url))
    }

    fn find_dep_mut(&mut self, url: &str) -> Option<&mut DependencyLock> {
        self.deps
            .iter_mut()
            .find(|l| l.url.eq_ignore_ascii_case(url))
    }
}

impl LoadableConfig<SpecLock> for SpecLock {
    fn lint(&mut self) {
        self.deps.sort_by(|a, b| a.url.cmp(&b.url));
        self.deps
            .dedup_by(|a, b| a.url.eq_ignore_ascii_case(&b.url));
    }
}

#[cfg(test)]
mod tests {

    use std::io::Write;

    use anyhow::Result;

    use super::*;
    use crate::core::tests;

    #[test]
    fn test_new_default_instance() {
        let sut = SpecLock::new();

        assert_eq!(
            VERSION, sut.version,
            "default instance version should be crate version"
        );
        assert_eq!(0, sut.deps.len(), "default instance should have no deps");
    }

    #[test]
    fn test_add_dependency() {
        let mut sut = SpecLock::new();
        let dep = DependencyLock {
            url: "some url".to_string(),
            refname: "some ref".to_string(),
        };

        sut.add(dep.clone());

        assert_eq!(1, sut.deps.len());
        assert_eq!(dep, sut.deps.first().unwrap().to_owned());
    }

    #[test]
    fn test_initialise_save_then_load() -> Result<()> {
        let tmp = tests::tempfile();
        let dep = DependencyLock {
            url: "some url".to_string(),
            refname: "some ref".to_string(),
        };
        let mut sut = SpecLock::new();
        sut.add(dep);

        sut.save_to(&tmp)?;
        let actual = SpecLock::load_from(&tmp)?;

        assert_eq!(sut, actual);

        Ok(())
    }

    #[test]
    fn test_cannot_load_invalid_file() -> Result<()> {
        let mut out = tempfile::NamedTempFile::new()?;
        out.write(b"bf")?;
        out.flush()?;

        let actual = SpecLock::load_from(out);
        assert!(actual.is_err(), "there should be an error");

        Ok(())
    }
}
