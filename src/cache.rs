use std::fs::create_dir_all;
use std::fs::remove_dir_all;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::format_err;
use anyhow::Result;
use sha2::Digest;

use crate::deps::Dependency;
use crate::lock::Lock;
use crate::preset::Preset;
use crate::repository::Repository;

pub struct Cache {
    path: PathBuf,
}

impl Cache {
    const LOCKS_DIR: &str = "locks";
    const REPOS_DIR: &str = "repos";

    pub fn new(preset: &Preset) -> Self {
        Self {
            path: preset.cache().into(),
        }
    }

    pub fn ensure(&self) -> Result<()> {
        create_dir_all(self.get_repos_path())
            .map_err(|err| format_err!("cannot create repos directory: {err}"))?;

        create_dir_all(self.get_locks_path())
            .map_err(|err| format_err!("cannot create locks directory: {err}"))?;

        Ok(())
    }

    pub fn lock(&self) -> Result<Lock> {
        let path = self.path.join(".LOCK");
        let mut lock = Lock::new(path).with_warn(
            "Cannot acquire cache lock, are yoy running a different instance in parallel?",
            Duration::from_secs(1),
        );
        lock.acquire()?;
        Ok(lock)
    }

    pub fn clean(&self) -> Result<()> {
        remove_dir_all(&self.path)
            .map_err(|err| format_err!("cannot remove cache directory: {err}"))?;

        Ok(())
    }

    pub fn get_repository(&self, dep: &Dependency) -> Result<Repository> {
        let path = self.get_repository_path(dep);
        let repo = Repository::new(path);
        repo.ensure(dep)
            .map_err(|err| format_err!("cannot ensure repository: {err}"))
    }

    fn get_repository_path(&self, dep: &Dependency) -> PathBuf {
        self.get_repos_path().join(Self::build_id(dep))
    }

    fn get_repository_lock_path(&self, dep: &Dependency) -> PathBuf {
        self.get_locks_path().join(Self::build_id(dep))
    }

    fn get_repos_path(&self) -> PathBuf {
        Path::new(&self.path).join(Self::REPOS_DIR)
    }

    fn get_locks_path(&self) -> PathBuf {
        Path::new(&self.path).join(Self::LOCKS_DIR)
    }

    fn build_id(dep: &Dependency) -> String {
        format!("{:x}", sha2::Sha256::digest(&dep.url))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_utils::build_preset;
    use crate::test_utils::TestContext;

    #[test]
    #[allow(unused_must_use)]
    fn test_cache_clean_and_ensure() {
        let context = &TestContext::new();
        let sut = Cache::new(&context.preset);

        let root: &PathBuf = &context.preset.cache().into();
        let repos = root.join("repos");
        let locks = root.join("locks");

        sut.clean();
        assert!(!root.exists());

        sut.ensure();
        assert!(root.exists());
        assert!(repos.exists());
        assert!(locks.exists());

        sut.clean();
        assert!(!root.exists());
    }

    #[test]
    fn test_cache_get_repository_path() {
        let preset = &build_preset();
        let dep = &Dependency::new("some-url", "some-branch");

        let sut = Cache::new(preset);

        assert_eq!(
            ".test-cache/repos/807460ee997e6fbe9d826f58a2af79c570f7bb5aa26f48d9b18dc320af428a05",
            sut.get_repository_path(dep).as_os_str()
        );
    }

    #[test]
    fn test_cache_get_repository_lock_path() {
        let preset = &build_preset();
        let dep = &Dependency::new("some-url", "some-branch");

        let sut = Cache::new(preset);

        assert_eq!(
            ".test-cache/locks/807460ee997e6fbe9d826f58a2af79c570f7bb5aa26f48d9b18dc320af428a05",
            sut.get_repository_lock_path(dep).as_os_str()
        );
    }
}
