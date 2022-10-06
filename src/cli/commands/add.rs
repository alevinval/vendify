use log::error;
use log::info;

use crate::core::Dependency;
use crate::core::LoadableConfig;
use crate::core::Spec;
use crate::VENDOR_YML;

pub fn run(
    url: &str,
    refname: &str,
    extensions: Option<Vec<String>>,
    targets: Option<Vec<String>>,
    ignores: Option<Vec<String>>,
) {
    let mut spec = match Spec::load_from(VENDOR_YML) {
        Ok(config) => config,
        Err(err) => {
            error!("{}", err);
            return;
        }
    };

    let mut dep = Dependency::new(url, refname);
    if let Some(extensions) = extensions {
        dep.filters.add_extensions(&extensions);
    }
    if let Some(targets) = targets {
        dep.filters.add_targets(&targets);
    }
    if let Some(ignores) = ignores {
        dep.filters.add_ignores(&ignores);
    }
    spec.add(dep);

    match spec.save_to(VENDOR_YML) {
        Ok(_) => {
            info!("added dependency {}@{}", url, refname);
        }
        Err(err) => {
            error!("add failed: {}", err)
        }
    }
}
