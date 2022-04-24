pub use dependency::Dependency;
pub use dependency_lock::DependencyLock;
pub use manager::DependencyManager;

#[allow(clippy::module_inception)]
mod dependency;
mod dependency_lock;
mod manager;
