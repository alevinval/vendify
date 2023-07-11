use std::fs::create_dir_all;
use std::fs::remove_dir_all;
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
    root: PathBuf,
    lock_file: PathBuf,
    locks_dir: PathBuf,
    repos_dir: PathBuf,
}

impl Cache {
    /// Creates a new [`Cache`] based on a [`Preset`].
    pub fn new(preset: &Preset) -> Self {
        let root = PathBuf::from(preset.cache());
        Self {
            lock_file: root.join(".LOCK"),
            locks_dir: root.join("locks"),
            repos_dir: root.join("repos"),
            root,
        }
    }

    /// Initializes the cache folder, making sure it exists and contains the expected
    /// directory structure.
    ///
    /// # Errors
    ///
    /// This function will return an error if something fails along the way.
    pub fn initialize(&self) -> Result<()> {
        create_dir_all(&self.repos_dir)
            .map_err(|err| format_err!("cannot create repos directory: {err}"))?;

        create_dir_all(&self.locks_dir)
            .map_err(|err| format_err!("cannot create locks directory: {err}"))?;

        Ok(())
    }

    /// Returns the lock of this [`Cache`]. It will log a warning message it takes
    /// longer than expected to acquire, as this could indicate the user is trying to run
    /// multiple vendoring processes at the same time, which is not supported, as the cache
    /// is shared.
    ///
    /// # Errors
    ///
    /// This function will return an error if the lock cannot be acquired.
    pub fn lock(&self) -> Result<Lock> {
        let mut lock = Lock::new(&self.lock_file).with_warn(
            "Cannot acquire cache lock, are you running a different instance in parallel?",
            Duration::from_secs(1),
        );
        lock.acquire()?;
        Ok(lock)
    }

    /// Returns the lock for a [`Dependency`] cache folder
    ///
    /// # Errors
    ///
    /// This function will return an error if the lock annot be acquired.
    pub fn lock_repository(&self, dep: &Dependency) -> Result<Lock> {
        let path = self.get_repository_lock_path(dep);
        let mut lock = Lock::new(path);
        lock.acquire()?;
        Ok(lock)
    }

    /// Removes the cache root directory.
    ///
    /// # Errors
    ///
    /// This function will return an error if the cache directory cannot be removed.
    pub fn clear(&self) -> Result<()> {
        remove_dir_all(&self.root)
            .map_err(|err| format_err!("cannot remove cache directory: {err}"))?;

        Ok(())
    }

    /// Returns a [`Repository`] from the cache directory.
    ///
    /// # Errors
    ///
    /// This function will return an error if cannot open repository.
    pub fn get_repository(&self, dep: &Dependency) -> Result<Repository> {
        let path = self.get_repository_path(dep);
        let repo = Repository::new(path);
        repo.ensure(dep)
            .map_err(|err| format_err!("cannot ensure repository: {err}"))
    }

    fn get_repository_path(&self, dep: &Dependency) -> PathBuf {
        self.repos_dir.join(url_md5(dep))
    }

    fn get_repository_lock_path(&self, dep: &Dependency) -> PathBuf {
        self.locks_dir.join(url_md5(dep))
    }
}

fn url_md5(dep: &Dependency) -> String {
    format!("{:x}", sha2::Sha256::digest(&dep.url))
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

        sut.clear();
        assert!(!root.exists());

        sut.initialize();
        assert!(root.exists());
        assert!(repos.exists());
        assert!(locks.exists());

        sut.clear();
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
