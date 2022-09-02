use once_cell::sync::OnceCell;
use std::env;
use std::path;
use std::path::Path;
use std::path::PathBuf;

type RootBuilderFn = fn() -> PathBuf;

pub struct CachePathBuilder {
    root: RootBuilderFn,
    cache: OnceCell<path::PathBuf>,
}

impl CachePathBuilder {
    pub fn new() -> Self {
        Self::new_in(env::temp_dir)
    }

    pub fn new_in(root_provider: RootBuilderFn) -> Self {
        CachePathBuilder {
            root: root_provider,
            cache: OnceCell::new(),
        }
    }

    pub fn get(&self) -> &path::Path {
        self.cache.get_or_init(|| self.build())
    }

    fn build(&self) -> PathBuf {
        (self.root)().join(".vendor-cli-cache").join("repositories")
    }
}

impl AsRef<Path> for CachePathBuilder {
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
        let provider = || Path::new("some").join("fake");

        let sut = CachePathBuilder::new_in(provider);
        let actual = sut.get();

        assert_eq!(
            (provider)().join(".vendor-cli-cache").join("repositories"),
            actual
        );
    }
}
