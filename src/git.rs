use std::fs::create_dir_all;
use std::fs::remove_dir_all;
use std::path::Path;

use anyhow::format_err;
use anyhow::Result;
use git2::build::RepoBuilder;
use git2::BranchType;
use git2::Config;
use git2::FetchOptions;
use git2::Oid;
use git2::RemoteCallbacks;
use git2::Repository;
use git2_credentials::CredentialHandler;
use log::error;
use log::info;

pub struct Git {}

impl Git {
    pub fn get_current_refname(&self, repository_path: &Path) -> Result<Oid> {
        match Repository::open(repository_path) {
            Ok(repository) => {
                let commit = repository
                    .head()
                    .map_err(|err| format_err!("cannot read current git HEAD: {}", err))?
                    .peel_to_commit()
                    .map_err(|err| format_err!("cannot read current git commit: {}", err))?;

                Ok(commit.id())
            }
            Err(err) => Err(err.into()),
        }
    }

    pub fn open_or_clone(&self, url: &str, refname: &str, repository_path: &Path) -> Result<()> {
        match Repository::open(repository_path) {
            Ok(_) => Ok(()),
            Err(_) => {
                if remove_dir_all(repository_path).is_ok() {
                    create_dir_all(repository_path)?;
                }
                match self.clone(url, refname, repository_path) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(format_err!(
                        "cannot load git repository from {path}: {err}",
                        path = repository_path.display(),
                        err = err,
                    )),
                }
            }
        }
    }

    pub fn clone(&self, url: &str, refname: &str, dst: &Path) -> Result<Repository> {
        info!("cloning {}...", url);

        let fetch_options = self.get_fetch_options()?;
        match RepoBuilder::new()
            .branch(refname)
            .fetch_options(fetch_options)
            .clone(url, dst)
        {
            Ok(it) => Ok(it),
            Err(err) => {
                error!("cannot clone {}: {}", url, err);
                Err(err.into())
            }
        }
    }

    pub fn checkout(&self, repository_path: &Path, refname: &str) -> Result<()> {
        let repository = Repository::open(repository_path)?;
        let (object, reference) = repository.revparse_ext(refname)?;
        repository.checkout_tree(&object, None)?;
        match reference {
            Some(gref) => {
                let name = gref
                    .name()
                    .expect("invalid reference, contains non-utf8 characters");
                repository.set_head(name)?;
            }
            None => {
                repository.set_head_detached(object.id())?;
            }
        }
        Ok(())
    }

    pub fn fetch(&self, repository_path: &Path, refname: &str) -> Result<()> {
        let repository = Repository::open(repository_path)?;
        let origin_refname = format!("origin/{}", refname);
        if let Err(err) = repository.find_branch(&origin_refname, BranchType::Remote) {
            return Err(format_err!("cannot find refname '{}': {}", refname, err));
        }
        let mut fo = self.get_fetch_options()?;
        repository
            .find_remote("origin")?
            .fetch(&[refname], Some(&mut fo), None)?;
        Ok(())
    }

    pub fn reset(&self, repository_path: &Path, refname: &str) -> Result<()> {
        let repository = Repository::open(repository_path)?;
        let oid = repository.refname_to_id(&format!("refs/remotes/origin/{}", refname))?;
        let object = repository.find_object(oid, None)?;
        repository.reset(&object, git2::ResetType::Hard, None)?;
        Ok(())
    }

    fn get_fetch_options(&self) -> Result<FetchOptions> {
        let config = match Config::open_default() {
            Ok(it) => it,
            Err(err) => {
                error!("cannot open git configuration: {err}", err = err);
                return Err(err.into());
            }
        };

        let mut credential_helper = CredentialHandler::new(config);

        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(move |url, username, allowed| {
            credential_helper.try_next_credential(url, username, allowed)
        });

        let mut fetch_options = FetchOptions::new();
        fetch_options
            .remote_callbacks(callbacks)
            .download_tags(git2::AutotagOption::All)
            .update_fetchhead(true);

        Ok(fetch_options)
    }
}
