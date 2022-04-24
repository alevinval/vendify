use crate::core::Dependency;
use once_cell::sync::OnceCell;
use sha2::Digest;
use std::path;
use std::path::Path;

pub struct RepositoryPathBuilder<'a> {
    dep: &'a Dependency,
    root: &'a Path,
    cache: OnceCell<path::PathBuf>,
}

impl<'a> RepositoryPathBuilder<'a> {
    pub fn new(dep: &'a Dependency, root: &'a Path) -> Self {
        RepositoryPathBuilder {
            dep,
            root,
            cache: OnceCell::new(),
        }
    }

    pub fn get(&self) -> &path::Path {
        self.cache.get_or_init(|| self.build_path())
    }

    fn build_path(&self) -> path::PathBuf {
        self.root.join(self.build_id())
    }

    fn build_id(&self) -> String {
        let mut hasher = sha2::Sha256::new();
        hasher.update(&self.dep.url);
        format!("{:x}", hasher.finalize())
    }
}

impl<'a> AsRef<Path> for RepositoryPathBuilder<'a> {
    fn as_ref(&self) -> &Path {
        self.get()
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

        let sut = RepositoryPathBuilder::new(&dep, &root);
        let actual = sut.get();

        assert_eq!(
            root.join("8f12b29c96078fd80e08a5d1d9c4ba5ddf5f76d0fa92bd3269f69cf3be6fc343"),
            actual
        );
    }
}
