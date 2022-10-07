use std::sync::Arc;
use std::sync::RwLock;

use log::error;
use log::info;

use crate::core::LoadableConfig;
use crate::core::Spec;
use crate::core::SpecInstaller;
use crate::core::SpecLock;
use crate::VENDOR_LOCK_YML;
use crate::VENDOR_YML;

pub fn run() {
    let spec = match Spec::load_from(VENDOR_YML) {
        Ok(value) => Arc::new(RwLock::new(value)),
        Err(err) => {
            error!("{}", err);
            return;
        }
    };

    let lock = match SpecLock::load_from(VENDOR_LOCK_YML) {
        Ok(value) => Arc::new(RwLock::new(value)),
        Err(_) => Arc::new(RwLock::new(SpecLock::new())),
    };

    let manager = SpecInstaller::new(Arc::clone(&spec), Arc::clone(&lock));
    if let Err(err) = manager.install() {
        error!("install failed: {}", err);
        return;
    };

    if let Err(err) = lock.write().unwrap().save_to(VENDOR_LOCK_YML) {
        error!("install failed: {}", err);
        return;
    }

    if let Err(err) = spec.write().unwrap().save_to(VENDOR_YML) {
        error!("install failed: {}", err);
        return;
    }

    info!("install success ✅")
}
