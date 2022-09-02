use log::debug;
use log::error;
use log::info;

use crate::core::CachePathFactory;
use crate::core::LoadableConfig;
use crate::core::VendorLock;
use crate::core::VendorManager;
use crate::core::VendorSpec;
use crate::VENDOR_LOCK_YML;
use crate::VENDOR_YML;

pub fn run() {
    let mut spec = match VendorSpec::load_from(VENDOR_YML) {
        Ok(value) => value,
        Err(err) => {
            error!("{}", err);
            return;
        }
    };

    let mut lock = match VendorLock::load_from(VENDOR_LOCK_YML) {
        Ok(value) => value,
        Err(_) => VendorLock::new(),
    };

    let cache = CachePathFactory::create_default();
    debug!("cache: {}", &cache.display());

    let mut manager = VendorManager::new(&cache, &mut spec, &mut lock);
    if let Err(err) = manager.install() {
        error!("install failed: {}", err);
        return;
    };

    if let Err(err) = lock.save_to(VENDOR_LOCK_YML) {
        error!("install failed: {}", err);
        return;
    }

    if let Err(err) = spec.save_to(VENDOR_YML) {
        error!("install failed: {}", err);
        return;
    }

    info!("install success ✅")
}
