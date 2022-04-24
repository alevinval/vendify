pub use self::vendor_spec::VendorManager;
pub use self::vendor_spec::VendorSpec;
pub use dependency::Dependency;
pub use dependency::DependencyLock;
pub use utils::LoadableConfig;
pub use vendor_lock::VendorLock;

mod dependency;
pub mod paths;
pub(crate) mod utils;
mod vendor_lock;
mod vendor_spec;
