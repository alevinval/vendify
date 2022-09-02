use std::path::Path;

use log::error;
use log::info;
use log::warn;

use crate::core::LoadableConfig;
use crate::core::VendorSpec;
use crate::VENDOR_YML;

pub fn run() {
    info!("initializing vendor in current directory");

    if Path::new(VENDOR_YML).exists() {
        warn!("{} already exists", VENDOR_YML);
        return;
    }

    let mut config = VendorSpec::new();
    if let Err(err) = config.save_to(VENDOR_YML) {
        error!("failed initializing: {}", err);
        return;
    }

    info!("{} has been created", VENDOR_YML);
}
