use std::path::Path;
use std::path::PathBuf;

use sha2::Digest;

use crate::core::Dependency;

pub struct RepositoryPathFactory {}

impl RepositoryPathFactory {
    pub fn create<P: AsRef<Path>>(dep: &Dependency, root: P) -> PathBuf {
        return root.as_ref().join(Self::build_id(&dep.url));
    }

    fn build_id(url: &str) -> String {
        let mut hasher = sha2::Sha256::new();
        hasher.update(url);
        format!("{:x}", hasher.finalize())
    }
}

#[cfg(test)]
mod tests {

    use std::path::Path;

    use super::*;

    #[test]
    fn test_get_path() {
        let dep = Dependency::new("http://some.url", "some-ref");
        let root = Path::new("some")
            .join("fake")
            .join(".vendor-cli-tool")
            .join("repositories");

        let actual = RepositoryPathFactory::create(&dep, &root);

        assert_eq!(
            root.join("8f12b29c96078fd80e08a5d1d9c4ba5ddf5f76d0fa92bd3269f69cf3be6fc343"),
            actual
        );
    }
}
