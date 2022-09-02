use crate::core::paths::PathIterator;
use crate::core::paths::RepositoryPathFactory;
use crate::core::utils::GitOps;
use crate::core::Dependency;
use crate::core::DependencyLock;
use crate::core::VendorSpec;
use anyhow::format_err;
use anyhow::Result;
use log::debug;
use log::info;
use log::warn;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

pub struct RepositoryContext<'a> {
    pub path_factory: &'a RepositoryPathFactory,
    pub path_iterator: &'a dyn PathIterator,
    pub git: &'a GitOps,
}

impl<'a> RepositoryContext<'a> {
    pub fn new(
        path_factory: &'a RepositoryPathFactory,
        path_iterator: &'a dyn PathIterator,
        git: &'a GitOps,
    ) -> Self {
        RepositoryContext {
            path_factory,
            path_iterator,
            git,
        }
    }
}

pub struct DependencyManager<'a> {
    spec: &'a VendorSpec,
    dependency: &'a Dependency,
    dependency_lock: Option<&'a DependencyLock>,
    context: &'a RepositoryContext<'a>,
}

impl<'a> DependencyManager<'a> {
    pub fn new(
        spec: &'a VendorSpec,
        dependency: &'a Dependency,
        dependency_lock: Option<&'a DependencyLock>,
        context: &'a RepositoryContext,
    ) -> Self {
        DependencyManager {
            spec,
            dependency,
            dependency_lock,
            context,
        }
    }

    /// Install copies the files of the dependency into the vendor folder.
    /// It respects the dependency lock, when passed.
    pub fn install<P: AsRef<Path>>(&self, to: P) -> Result<DependencyLock> {
        self.ensure_repository()?;
        let refname = self.get_locked_refname();

        info!("installing {}@{}", self.dependency.url, refname);
        self.context
            .git
            .checkout(self.context.path_factory.create(), refname)?;
        self.import(to)
    }

    /// Update fetches latest changes from the git remote, against the
    /// reference. Then it installs the dependency. This will ignore the
    /// lock file and generate a new lock with the updated reference.
    pub fn update<P: AsRef<Path>>(&self, to: P) -> Result<DependencyLock> {
        self.ensure_repository()?;
        let refname = &self.dependency.refname;

        info!("updating {}@{}", self.dependency.url, refname);
        self.context
            .git
            .fetch(self.context.path_factory.create(), refname)?;
        self.context
            .git
            .reset(self.context.path_factory.create(), refname)?;
        self.import(to)
    }

    fn import<P: AsRef<Path>>(&self, dst_root: P) -> Result<DependencyLock> {
        self.copy_files(dst_root)?;
        let locked = self.get_locked_dependency()?;
        info!("\t🔒 {}", locked.refname);
        Ok(locked)
    }

    fn copy_files<P: AsRef<Path>>(&self, dst_root: P) -> Result<(), anyhow::Error> {
        let root = self.context.path_factory.create();
        for src_path in self.context.path_iterator.iter() {
            let relative_path = src_path.strip_prefix(root)?;
            if self.is_ignored(relative_path) {
                warn!("\t- {} [IGNORED]", relative_path.display());
                continue;
            }
            if !self.is_target(relative_path) {
                debug!("\t- {} [NOT TARGET]", relative_path.display());
                continue;
            }
            if !self.is_extension(relative_path) {
                debug!("\t- {} [IGNORED EXTENSION]", relative_path.display());
                continue;
            }

            let dst_path = dst_root.as_ref().join(relative_path);
            info!(
                "\t.../{} -> {}",
                relative_path.display(),
                dst_path.display()
            );
            copy_file(src_path, dst_path)?;
        }
        Ok(())
    }

    fn get_locked_refname(&self) -> &str {
        match self.dependency_lock {
            Some(it) => &it.refname,
            None => &self.dependency.refname,
        }
    }

    fn get_locked_dependency(&self) -> Result<DependencyLock> {
        let refname = self
            .context
            .git
            .get_current_refname(self.context.path_factory.create())?;
        Ok(DependencyLock {
            url: self.dependency.url.clone(),
            refname: refname.to_string(),
        })
    }

    /// Returns the repository from the local cache.
    /// If the repository does not exist, it will clone it.
    fn ensure_repository(&self) -> Result<()> {
        let repository_path = self.context.path_factory.create();
        let result = self.context.git.open_or_clone(
            &self.dependency.url,
            &self.dependency.refname,
            repository_path,
        );

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(format_err!("cannot open repository: {}", err)),
        }
    }

    fn is_ignored(&self, path: &Path) -> bool {
        return chained_any(
            &self.spec.ignores,
            &self.dependency.ignores,
            &path_matcher(path),
        );
    }

    fn is_target(&self, path: &Path) -> bool {
        return chained_any(
            &self.spec.targets,
            &self.dependency.targets,
            &path_matcher(path),
        );
    }

    fn is_extension(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            return chained_any(
                &self.spec.extensions,
                &self.dependency.extensions,
                &extension_matcher(ext),
            );
        }
        false
    }
}

fn path_matcher(path: &Path) -> Box<dyn Fn(&String) -> bool + '_> {
    Box::new(|base| path.starts_with(base))
}

fn extension_matcher(input: &OsStr) -> Box<dyn Fn(&String) -> bool + '_> {
    Box::new(|ext| input.eq_ignore_ascii_case(ext))
}

fn chained_any(a: &[String], b: &[String], matcher: &dyn Fn(&String) -> bool) -> bool {
    return a.iter().chain(b.iter()).any(matcher);
}

fn copy_file<P: AsRef<Path>>(from: P, to: P) -> Result<()> {
    if let Some(parent) = to.as_ref().parent() {
        fs::create_dir_all(parent)?
    };
    fs::copy(from, to)?;
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::core::utils::tests;

    #[test]
    fn test_copy_file_when_dst_parent_does_not_exit() -> Result<()> {
        let from = tests::tempdir().path().join("src/path/file.txt");
        fs::create_dir_all(from.parent().unwrap())?;
        tests::write_to(&from, "some-file");

        let to = tests::tempdir().path().join("dst/parent/file.txt");

        assert!(!to.exists());
        copy_file(&from, &to)?;
        assert!(to.exists());

        let contents = tests::read_as_str(to);
        assert_eq!("some-file", contents);

        Ok(())
    }

    #[test]
    fn test_copy_file_when_dst_parent_exists() -> Result<()> {
        let from = tests::tempdir().path().join("src/path/file.txt");
        fs::create_dir_all(from.parent().unwrap())?;
        tests::write_to(&from, "some-file");

        let to = tests::tempdir().path().join("file.txt");

        assert!(!to.exists());
        copy_file(&from, &to)?;
        assert!(to.exists());

        let contents = tests::read_as_str(to);
        assert_eq!("some-file", contents);

        Ok(())
    }
}
