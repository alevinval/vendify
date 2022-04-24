use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Dependency {
    pub url: String,
    pub refname: String,
    pub extensions: Vec<String>,
    pub targets: Vec<String>,
    pub ignores: Vec<String>,
}

impl Dependency {
    /// Creates a new dependency configuration, uses sane default values, which
    /// come pre-configured for working with proto files.
    pub fn new(url: &str, refname: &str) -> Self {
        Dependency {
            url: url.to_string(),
            refname: refname.to_string(),
            extensions: vec![],
            targets: vec![],
            ignores: vec![],
        }
    }

    /// Updates the values, taken from another dependency.
    pub fn update_config(&mut self, other: &Dependency) {
        self.refname = other.refname.clone();
        self.extensions = other.extensions.clone();
        self.targets = other.targets.clone();
        self.ignores = other.ignores.clone();
    }
}
