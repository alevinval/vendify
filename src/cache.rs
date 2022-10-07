use std::fs::create_dir_all;
use std::fs::remove_dir_all;
use std::path::Path;
use std::path::PathBuf;

use anyhow::format_err;
use anyhow::Result;
use sha2::Digest;

use crate::dependency::Dependency;
use crate::repository::Repository;

const REPOS_DIR: &str = "repos";
const LOCKS_DIR: &str = "locks";

pub struct CacheManager {
    cache_dir: String,
}

impl CacheManager {
    pub fn new() -> Self {
        CacheManager {
            cache_dir: ".vendify".to_owned(),
        }
    }

    pub fn ensure(&self) -> Result<()> {
        create_dir_all(self.get_repos_path())
            .map_err(|err| format_err!("cannot create repos directory: {:?}", err))?;

        create_dir_all(self.get_locks_path())
            .map_err(|err| format_err!("cannot create locks directory: {:?}", err))?;

        Ok(())
    }

    pub fn clean(&self) -> Result<()> {
        remove_dir_all(&self.cache_dir)
            .map_err(|err| format_err!("cannot remove cache directory: {:?}", err))?;

        Ok(())
    }

    pub fn get_repository(&self, dep: &Dependency) -> Result<Repository> {
        let path = self.get_repository_path(dep);
        let repo = Repository::new(path);
        repo.ensure(dep)
            .map_err(|err| format_err!("cannot ensure repository: {:?}", err))
    }

    fn get_repository_path(&self, dep: &Dependency) -> PathBuf {
        self.get_repos_path().join(get_dependency_id(dep))
    }

    fn get_repository_lock_path(&self, dep: &Dependency) -> PathBuf {
        self.get_locks_path().join(get_dependency_id(dep))
    }

    fn get_repos_path(&self) -> PathBuf {
        Path::new(&self.cache_dir).join(REPOS_DIR)
    }

    fn get_locks_path(&self) -> PathBuf {
        Path::new(&self.cache_dir).join(LOCKS_DIR)
    }
}

fn get_dependency_id(dep: &Dependency) -> String {
    format!("{:x}", sha2::Sha256::digest(&dep.url))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::CacheManager;
    use crate::dependency::Dependency;

    #[test]
    #[allow(unused_must_use)]
    fn test_cache_manager_clean_and_ensure() {
        let sut = CacheManager::new();

        let root = Path::new(".vendor-rs");
        let repos = root.join("repos");
        let locks = root.join("locks");

        sut.clean();
        assert_eq!(false, root.exists());

        sut.ensure();
        assert_eq!(true, root.exists());
        assert_eq!(true, repos.exists());
        assert_eq!(true, locks.exists());

        sut.clean();
        assert_eq!(false, root.exists());
    }

    #[test]
    fn test_cache_manager_get_repository_path() {
        let dep = &Dependency::new("some-url", "some-branch");

        let sut = CacheManager::new();

        assert_eq!(
            ".vendor-rs/repos/807460ee997e6fbe9d826f58a2af79c570f7bb5aa26f48d9b18dc320af428a05",
            sut.get_repository_path(dep).as_os_str()
        );
    }

    #[test]
    fn test_cache_manager_get_repository_lock_path() {
        let dep = &Dependency::new("some-url", "some-branch");

        let sut = CacheManager::new();

        assert_eq!(
            ".vendor-rs/locks/807460ee997e6fbe9d826f58a2af79c570f7bb5aa26f48d9b18dc320af428a05",
            sut.get_repository_lock_path(dep).as_os_str()
        );
    }
}
