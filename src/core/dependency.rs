use std::fmt::Debug;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct Dependency {
    pub url: String,
    pub refname: String,

    #[serde(default = "Vec::new")]
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub extensions: Vec<String>,

    #[serde(default = "Vec::new")]
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub targets: Vec<String>,

    #[serde(default = "Vec::new")]
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub ignores: Vec<String>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct DependencyLock {
    pub url: String,
    pub refname: String,
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
    pub fn update_from(&mut self, other: &Dependency) {
        self.refname = other.refname.clone();
        self.extensions = other.extensions.clone();
        self.targets = other.targets.clone();
        self.ignores = other.ignores.clone();
    }
}

#[cfg(test)]
mod tests {

    use super::Dependency;

    #[test]
    fn test_dependency_update_from() {
        let mut original = Dependency::new("url-a", "refname-a");
        original.extensions = vec!["a".into()];
        original.targets = vec!["b".into()];
        original.ignores = vec!["c".into()];

        let mut other = Dependency::new("url-b", "refname-b");
        other.extensions = vec!["1".into()];
        other.targets = vec!["2".into()];
        other.ignores = vec!["3".into()];

        let mut actual = original.clone();
        actual.update_from(&other);

        assert_eq!("refname-b", actual.refname);
        assert_eq!(vec!["1".to_string()], actual.extensions);
        assert_eq!(vec!["2".to_string()], actual.targets);
        assert_eq!(vec!["3".to_string()], actual.ignores);
    }
}
