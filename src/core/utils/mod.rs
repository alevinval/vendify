pub use git_ops::GitOps;
pub use loadable_config::LoadableConfig;
use std::path::Path;

mod git_ops;
mod loadable_config;
pub mod tests;

pub trait PrefixVec {
    fn matching_prefix(&self, path: &Path) -> Option<&String>;
    fn any_matching_prefix(&self, path: &Path) -> bool;
}

impl PrefixVec for Vec<String> {
    fn matching_prefix(&self, path: &Path) -> Option<&String> {
        self.iter().find(|prefix| path.starts_with(prefix))
    }

    fn any_matching_prefix(&self, path: &Path) -> bool {
        self.iter().any(|prefix| path.starts_with(prefix))
    }
}

impl PrefixVec for Option<Vec<String>> {
    fn matching_prefix(&self, path: &Path) -> Option<&String> {
        match self {
            Some(v) => v.matching_prefix(path),
            None => None,
        }
    }

    fn any_matching_prefix(&self, path: &Path) -> bool {
        match self {
            Some(v) => v.any_matching_prefix(path),
            None => false,
        }
    }
}
