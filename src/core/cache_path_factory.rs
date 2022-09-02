use std::env;
use std::path::PathBuf;

type RootBuilderFn = fn() -> PathBuf;

pub struct CachePathFactory {}

impl CachePathFactory {
    pub fn create_default() -> PathBuf {
        Self::create(env::temp_dir)
    }

    pub fn create(root_builder: RootBuilderFn) -> PathBuf {
        (root_builder)()
            .join(".vendor-cli-cache")
            .join("repositories")
    }
}

#[cfg(test)]
mod tests {

    use std::path::Path;

    use super::*;

    #[test]
    fn test_get_path() {
        let provider = || Path::new("some").join("fake");

        let actual = CachePathFactory::create(provider);

        assert_eq!(
            (provider)().join(".vendor-cli-cache").join("repositories"),
            actual
        );
    }
}
