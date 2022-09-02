use crate::core::Dependency;
use crate::core::LoadableConfig;
use crate::VERSION;
use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct VendorSpec {
    /// Version that was used to generate the spec
    pub version: String,

    /// Vendor directory path
    #[serde(default = "default_vendor")]
    #[serde(skip_serializing_if = "is_default_vendor")]
    pub vendor: String,

    #[serde(default = "Vec::new")]
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub extensions: Vec<String>,

    #[serde(default = "Vec::new")]
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub targets: Vec<String>,

    #[serde(default = "Vec::new")]
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub ignores: Vec<String>,

    /// List of dependencies
    pub deps: Vec<Dependency>,

    /// Last time the configuration was updated
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

impl VendorSpec {
    pub fn new() -> Self {
        VendorSpec {
            version: VERSION.to_string(),
            vendor: default_vendor(),
            extensions: Vec::new(),
            targets: Vec::new(),
            ignores: Vec::new(),
            deps: Vec::new(),
            updated_at: Utc::now(),
        }
    }

    pub fn add(&mut self, dep: Dependency) {
        match self.find_dep(&dep) {
            Some(existing) => existing.update_from(&dep),
            None => self.deps.push(dep),
        }
        self.updated_at = Utc::now();
    }

    fn find_dep(&mut self, dep: &Dependency) -> Option<&mut Dependency> {
        self.deps
            .iter_mut()
            .find(|d| d.url.eq_ignore_ascii_case(&dep.url))
    }
}

impl LoadableConfig<VendorSpec> for VendorSpec {
    fn lint(&mut self) {
        self.deps.sort_by(|a, b| a.url.cmp(&b.url));
        self.deps
            .dedup_by(|a, b| a.url.eq_ignore_ascii_case(&b.url));
    }
}

fn default_vendor() -> String {
    "vendor".to_string()
}

fn is_default_vendor(other: &str) -> bool {
    other.eq_ignore_ascii_case(&default_vendor())
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::core::utils::tests;
    use anyhow::Result;
    use std::io::Write;

    #[test]
    fn test_new_default_instance() {
        let sut = VendorSpec::new();

        assert_eq!(
            VERSION, sut.version,
            "default instance version should be crate version"
        );
        assert_eq!(0, sut.deps.len(), "default instance should have no deps");
    }

    #[test]
    fn test_add_dependency() {
        let mut sut = VendorSpec::new();
        let dep = Dependency::new("some url", "some ref");

        sut.add(dep.clone());

        assert_eq!(1, sut.deps.len());
        assert_eq!(dep, sut.deps.first().unwrap().to_owned());
    }

    #[test]
    fn test_initialise_save_then_load() -> Result<()> {
        let tmp = tests::tempfile();
        let dep = Dependency::new("some url", "some ref");
        let mut sut = VendorSpec::new();
        sut.add(dep);

        sut.save_to(&tmp)?;
        let actual = VendorSpec::load_from(&tmp)?;

        assert_eq!(sut, actual);

        Ok(())
    }

    #[test]
    fn test_cannot_load_invalid_file() -> Result<()> {
        let mut out = tempfile::NamedTempFile::new()?;
        out.write(b"bf")?;
        out.flush()?;

        let actual = VendorSpec::load_from(out);
        assert!(actual.is_err(), "there should be an error");

        Ok(())
    }
}
