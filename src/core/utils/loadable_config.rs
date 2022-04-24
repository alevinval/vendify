use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs;
use std::path::Path;

pub trait LoadableConfig<T>
where
    T: Sized + Serialize + DeserializeOwned,
    Self: Serialize,
{
    fn load_from<P: AsRef<Path>>(path: P) -> Result<T> {
        let load = || -> Result<T> {
            let f = fs::File::open(&path)?;
            let config = serde_yaml::from_reader(&f)?;
            Ok(config)
        };

        match load() {
            Ok(config) => Ok(config),
            Err(err) => Err(anyhow::format_err!(
                "cannot load {path}: {err}",
                path = path.as_ref().display(),
                err = err
            )),
        }
    }

    fn save_to<P: AsRef<Path>>(&mut self, out: P) -> Result<&Self> {
        let mut save = || -> Result<()> {
            self.lint();
            let contents = serde_yaml::to_string(&self)?;
            fs::write(&out, contents)?;
            Ok(())
        };

        match save() {
            Ok(_) => Ok(self),
            Err(err) => Err(anyhow::format_err!(
                "cannot save {path}: {err}",
                path = out.as_ref().display(),
                err = err
            )),
        }
    }

    fn lint(&mut self);
}
