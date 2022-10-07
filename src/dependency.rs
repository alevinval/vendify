use std::fmt::Debug;

use serde::Deserialize;
use serde::Serialize;

use crate::filters::Filters;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct Dependency {
    pub url: String,
    pub refname: String,

    #[serde(flatten)]
    pub filters: Filters,
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
            filters: Filters::new(),
        }
    }

    /// Updates the values, taken from another dependency.
    pub fn update_from(&mut self, other: &Dependency) {
        self.refname = other.refname.clone();
        self.filters = other.filters.clone();
    }
}

#[cfg(test)]
mod tests {

    use super::Dependency;

    #[test]
    fn test_dependency_update_from() {
        let mut original = Dependency::new("url-a", "refname-a");
        original
            .filters
            .add_extensions(&vec!["a".into()])
            .add_targets(&vec!["b".into()])
            .add_ignores(&vec!["c".into()]);

        let mut other = Dependency::new("url-b", "refname-b");
        other
            .filters
            .add_extensions(&vec!["1".into()])
            .add_targets(&vec!["2".into()])
            .add_ignores(&vec!["3".into()]);

        let mut actual = original.clone();
        actual.update_from(&other);

        assert_eq!("refname-b", actual.refname);
        assert_eq!(actual.filters, other.filters);
    }
}
