use super::dependency::DependencyLock;
use super::LoadableConfig;
use crate::VERSION;
use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct VendorLock {
    /// Version that was used to generate the config
    pub version: String,

    /// List of locked dependencies
    pub deps: Vec<DependencyLock>,

    /// Last time the configuration was updated
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

impl VendorLock {
    pub fn new() -> Self {
        VendorLock {
            version: VERSION.to_owned(),
            deps: Vec::new(),
            updated_at: Utc::now(),
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
        self.updated_at = Utc::now();
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

impl LoadableConfig<VendorLock> for VendorLock {
    fn lint(&mut self) {
        self.deps.sort_by(|a, b| a.url.cmp(&b.url));
        self.deps
            .dedup_by(|a, b| a.url.eq_ignore_ascii_case(&b.url));
    }
}
