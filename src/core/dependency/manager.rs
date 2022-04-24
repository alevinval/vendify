use super::dependency::Dependency;
use crate::core::paths::PathCollector;
use crate::core::paths::RepositoryPathBuilder;
use crate::core::utils::GitOps;
use crate::core::utils::PrefixVec;
use crate::core::DependencyLock;
use anyhow::format_err;
use anyhow::Result;
use git2::Repository;
use log::debug;
use log::info;
use log::warn;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

pub struct DependencyManager<'imp> {
    cache: &'imp Path,
    dependency: &'imp Dependency,
    dependency_lock: Option<&'imp DependencyLock>,
}

impl<'imp> DependencyManager<'imp> {
    pub fn new(
        cache: &'imp Path,
        dependency: &'imp Dependency,
        dependency_lock: Option<&'imp DependencyLock>,
    ) -> Self {
        DependencyManager {
            cache,
            dependency,
            dependency_lock,
        }
    }

    /// Install copies the files of the dependency into the vendor folder.
    /// It respects the dependency lock, when passed.
    pub fn install<P: AsRef<Path>>(&self, to: P) -> Result<DependencyLock> {
        let repository = self.repository(self.cache)?;
        let refname = self.get_locked_refname();

        info!("installing {}@{}", self.dependency.url, refname);
        GitOps::checkout(&repository, refname)?;
        self.import(&repository, to)
    }

    /// Update fetches latest changes from the git remote, against the
    /// reference. Then it installs the dependency. This will ignore the
    /// lock file and generate a new lock with the updated reference.
    pub fn update<P: AsRef<Path>>(&self, to: P) -> Result<DependencyLock> {
        let repository = self.repository(self.cache)?;
        let refname = &self.dependency.refname;

        info!("updating {}@{}", self.dependency.url, refname);
        GitOps::fetch(&repository, refname)?;
        GitOps::reset(&repository, refname)?;
        self.import(&repository, to)
    }

    fn import<P: AsRef<Path>>(&self, repository: &Repository, to: P) -> Result<DependencyLock> {
        let root = repository.workdir().expect("cannot open repository");
        let collector = PathCollector::new(&root);
        for src in collector.iter() {
            let src_without_root = src.strip_prefix(root)?;
            if self
                .dependency
                .ignores
                .any_matching_prefix(src_without_root)
            {
                warn!("\t- {} [IGNORED]", src_without_root.display());
                continue;
            }
            if !self.dependency.targets.is_empty()
                && !self
                    .dependency
                    .targets
                    .any_matching_prefix(src_without_root)
            {
                debug!("\t- {} [NOT TARGET]", src_without_root.display());
                continue;
            }
            if let Some(target) = self.dependency.targets.matching_prefix(src_without_root) {
                if Path::new(target) != src_without_root
                    && !self.has_valid_extension(src_without_root)
                {
                    continue;
                }

                let dst = to.as_ref().join(src_without_root);
                info!("\t.../{} -> {}", src_without_root.display(), dst.display());
                import_file(src, dst)?;
            }
        }

        let locked = self.create_dependency_lock(repository)?;
        info!("\t🔒 {}", locked.refname);

        Ok(locked)
    }

    fn get_locked_refname(&self) -> &str {
        match self.dependency_lock {
            Some(it) => &it.refname,
            None => &self.dependency.refname,
        }
    }

    fn create_dependency_lock(&self, repository: &Repository) -> Result<DependencyLock> {
        let commit = GitOps::get_current_commit(repository)?;
        Ok(DependencyLock {
            url: self.dependency.url.clone(),
            refname: commit.id().to_string(),
        })
    }

    /// Returns the repository from the local cache. If the repository does not
    /// exist, it will clone it.
    fn repository(&self, cache_root: &Path) -> Result<Repository> {
        let builder = RepositoryPathBuilder::new(self.dependency, cache_root);
        let repository_path = builder.get();
        let repository = GitOps::open_or_clone(
            &self.dependency.url,
            &self.dependency.refname,
            repository_path,
        );

        match repository {
            Ok(it) => Ok(it),
            Err(err) => Err(format_err!("cannot open repository: {}", err)),
        }
    }

    fn has_valid_extension(&self, path: &Path) -> bool {
        if self.dependency.extensions.is_empty() {
            return true;
        }
        let extension = match path.extension() {
            Some(it) => it,
            None => return false,
        };
        return self
            .dependency
            .extensions
            .iter()
            .any(|e| OsStr::new(e).eq_ignore_ascii_case(extension));
    }
}

fn import_file<P: AsRef<Path>>(from: P, to: P) -> Result<()> {
    if let Some(parent) = to.as_ref().parent() {
        fs::create_dir_all(parent)?
    };
    fs::copy(from, to)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::core::utils::tests::*;

    use super::*;

    #[test]
    fn test_import_file_when_dst_parent_does_not_exit() -> Result<()> {
        let from = tempdir().path().join("src/path/file.txt");
        fs::create_dir_all(from.parent().unwrap())?;
        write_to(&from, "some-file");

        let to = tempdir().path().join("dst/parent/file.txt");

        assert!(!to.exists());
        import_file(&from, &to)?;
        assert!(to.exists());

        let contents = read_as_str(to);
        assert_eq!("some-file", contents);

        Ok(())
    }

    #[test]
    fn test_import_file_when_dst_parent_exists() -> Result<()> {
        let from = tempdir().path().join("src/path/file.txt");
        fs::create_dir_all(from.parent().unwrap())?;
        write_to(&from, "some-file");

        let to = tempdir().path().join("file.txt");

        assert!(!to.exists());
        import_file(&from, &to)?;
        assert!(to.exists());

        let contents = read_as_str(to);
        assert_eq!("some-file", contents);

        Ok(())
    }
}
