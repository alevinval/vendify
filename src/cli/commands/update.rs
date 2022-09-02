use crate::core::paths::CachePathBuilder;
use crate::core::LoadableConfig;
use crate::core::VendorLock;
use crate::core::VendorManager;
use crate::core::VendorSpec;
use crate::VENDOR_LOCK_YML;
use crate::VENDOR_YML;
use log::debug;
use log::error;
use log::info;

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

    // let vendor_dir = Path::new("vendor");
    // if vendor_dir.exists() {
    //     fs::remove_dir_all(vendor_dir).expect("cannot delete vendor folder");
    // }

    let cache = CachePathBuilder::new();
    debug!("cache: {}", cache.get().display());

    let mut manager = VendorManager::new(&cache, &mut spec, &mut lock);
    if let Err(err) = manager.update() {
        error!("update failed: {}", err);
        return;
    };

    if let Err(err) = lock.save_to(VENDOR_LOCK_YML) {
        error!("update failed: {}", err);
        return;
    }

    if let Err(err) = spec.save_to(VENDOR_YML) {
        error!("update failed: {}", err);
        return;
    }

    info!("update success ✅")
}
