pub use self::spec::VendorSpec;
pub use dependency::Dependency;
pub use dependency::DependencyLock;
pub use managers::VendorManager;
pub use spec_lock::VendorLock;
pub use utils::LoadableConfig;

mod dependency;
mod managers;
pub mod paths;
mod spec;
mod spec_lock;
pub(crate) mod utils;
