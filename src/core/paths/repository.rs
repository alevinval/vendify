use crate::core::Dependency;
use once_cell::sync::OnceCell;
use sha2::Digest;
use std::path::Path;
use std::path::PathBuf;

pub struct RepositoryPathFactory {
    url: String,
    root: PathBuf,
    cache: OnceCell<PathBuf>,
}

impl RepositoryPathFactory {
    pub fn new<P: AsRef<Path>>(dep: &Dependency, root: P) -> Self {
        RepositoryPathFactory {
            url: dep.url.to_owned(),
            root: root.as_ref().to_owned(),
            cache: OnceCell::new(),
        }
    }

    pub fn create(&self) -> &Path {
        self.cache.get_or_init(|| self.build_path())
    }

    fn build_path(&self) -> PathBuf {
        self.root.join(self.build_id())
    }

    fn build_id(&self) -> String {
        let mut hasher = sha2::Sha256::new();
        hasher.update(&self.url);
        format!("{:x}", hasher.finalize())
    }
}

impl AsRef<Path> for RepositoryPathFactory {
    fn as_ref(&self) -> &Path {
        self.create()
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

        let sut = RepositoryPathFactory::new(&dep, &root);
        let actual = sut.create();

        assert_eq!(
            root.join("8f12b29c96078fd80e08a5d1d9c4ba5ddf5f76d0fa92bd3269f69cf3be6fc343"),
            actual
        );
    }
}
