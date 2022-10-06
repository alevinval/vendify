use std::sync::Arc;
use std::sync::RwLock;

use log::debug;
use log::error;
use log::info;

use crate::core::CachePathFactory;
use crate::core::LoadableConfig;
use crate::core::Spec;
use crate::core::SpecLock;
use crate::core::VendorManager;
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

    // let vendor_dir = Path::new("vendor");
    // if vendor_dir.exists() {
    //     fs::remove_dir_all(vendor_dir).expect("cannot delete vendor folder");
    // }

    let cache = CachePathFactory::create_default();
    debug!("cache: {}", &cache.display());

    let manager = VendorManager::new(&cache, Arc::clone(&spec), Arc::clone(&lock));
    if let Err(err) = manager.update() {
        error!("update failed: {}", err);
        return;
    };

    if let Err(err) = lock.write().unwrap().save_to(VENDOR_LOCK_YML) {
        error!("update failed: {}", err);
        return;
    }

    if let Err(err) = spec.write().unwrap().save_to(VENDOR_YML) {
        error!("update failed: {}", err);
        return;
    }

    info!("update success ✅")
}
