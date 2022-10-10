use std::fs;
use std::path::Path;

use anyhow::format_err;
use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn load<T: Sized + DeserializeOwned, P: AsRef<Path>>(path: P) -> Result<T> {
    let do_load = || -> Result<T> {
        let f = fs::File::open(&path)?;
        let config = serde_yaml::from_reader(&f)?;
        Ok(config)
    };

    match do_load() {
        Ok(config) => Ok(config),
        Err(err) => Err(format_err!(
            "cannot load {path}: {err}",
            path = path.as_ref().display(),
            err = err
        )),
    }
}

pub fn save<T: Sized + Serialize, P: AsRef<Path>>(input: &T, path: P) -> Result<()> {
    let do_save = || -> Result<()> {
        let contents = serde_yaml::to_string(input)?;
        fs::write(&path, contents)?;
        Ok(())
    };

    match do_save() {
        Ok(_) => Ok(()),
        Err(err) => Err(format_err!(
            "cannot save {path}: {err}",
            path = path.as_ref().display(),
            err = err
        )),
    }
}
