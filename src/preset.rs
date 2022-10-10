use std::env::temp_dir;
use std::fmt;

use log::warn;

use crate::deps::Dependency;
use crate::filters::Filters;

type DependencyFiltersProvider = fn(&Dependency) -> Filters;

pub struct Preset {
    name: String,
    cache: String,
    vendor: String,
    spec: String,
    spec_lock: String,
    force_filters: bool,
    spec_filters: Filters,
    dependency_filters: DependencyFiltersProvider,
}

impl PartialEq for Preset {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.cache == other.cache
            && self.vendor == other.vendor
            && self.spec == other.spec
            && self.spec_lock == other.spec_lock
            && self.force_filters == other.force_filters
            && self.spec_filters == other.spec_filters
            && self.dependency_filters as usize == other.dependency_filters as usize
    }
}

impl Eq for Preset {}

impl fmt::Debug for Preset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Preset")
            .field("name", &self.name)
            .field("vendor", &self.vendor)
            .field("cache", &self.cache)
            .field("spec", &self.spec)
            .field("spec_lock", &self.spec_lock)
            .field("spec_filters", &self.spec_filters)
            .field("force_filters", &self.force_filters)
            .finish()
    }
}

impl Preset {
    #[must_use]
    pub fn new() -> Self {
        Builder::new().build()
    }

    #[must_use]
    pub fn name(&self) -> &String {
        &self.name
    }

    #[must_use]
    pub fn cache(&self) -> &String {
        &self.cache
    }

    #[must_use]
    pub fn vendor(&self) -> &String {
        &self.vendor
    }

    #[must_use]
    pub fn spec(&self) -> &String {
        &self.spec
    }

    #[must_use]
    pub fn spec_lock(&self) -> &String {
        &self.spec_lock
    }

    #[must_use]
    pub fn global_filters(&self) -> Filters {
        self.spec_filters.clone()
    }

    #[must_use]
    pub fn dependency_filters(&self, dep: &Dependency) -> Filters {
        (self.dependency_filters)(dep)
    }

    #[must_use]
    pub fn force_filters(&self) -> bool {
        self.force_filters
    }
}

impl Default for Preset {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Builder {
    name: String,
    cache: String,
    vendor: String,
    spec: String,
    spec_lock: String,
    force_filters: bool,
    global_filters: Filters,
    dependency_filters: DependencyFiltersProvider,
}

impl Builder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: "default".to_string(),
            cache: Self::default_cache(),
            vendor: "vendor".into(),
            spec: ".vendor.yml".into(),
            spec_lock: ".vendor-lock.yml".into(),
            force_filters: false,
            global_filters: Filters::new(),
            dependency_filters: Self::default_dependency_filters,
        }
    }

    #[must_use]
    pub fn build(self) -> Preset {
        Preset {
            name: self.name,
            vendor: self.vendor,
            spec: self.spec,
            spec_lock: self.spec_lock,
            cache: self.cache,
            spec_filters: self.global_filters,
            dependency_filters: self.dependency_filters,
            force_filters: self.force_filters,
        }
    }

    #[must_use]
    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    #[must_use]
    pub fn cache(mut self, path: &str) -> Self {
        self.cache = path.into();
        self
    }

    #[must_use]
    pub fn vendor(mut self, path: &str) -> Self {
        self.vendor = path.into();
        self
    }

    #[must_use]
    pub fn spec(mut self, path: &str) -> Self {
        self.spec = path.into();
        self
    }

    #[must_use]
    pub fn spec_lock(mut self, path: &str) -> Self {
        self.spec_lock = path.into();
        self
    }

    #[must_use]
    pub fn global_filters(mut self, filters: Filters) -> Self {
        self.global_filters = filters;
        self
    }

    #[must_use]
    pub fn dependency_filters(mut self, provider: DependencyFiltersProvider) -> Self {
        self.dependency_filters = provider;
        self
    }

    #[must_use]
    pub fn force_filters(mut self, force: bool) -> Self {
        self.force_filters = force;
        self
    }

    fn default_cache() -> String {
        let root = if let Some(home) = home::home_dir() {
            home
        } else {
            warn!("Cannot find user home directory, using tempdir as home");
            temp_dir()
        };

        root.join(".vendify")
            .into_os_string()
            .into_string()
            .unwrap_or_else(|_| ".vendify".into())
    }

    fn default_dependency_filters(_: &Dependency) -> Filters {
        Filters::new()
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_default_preset_equals_itself() {
        assert_eq!(Preset::default(), Preset::default());
    }

    #[test]
    fn test_default_preset_not_equals_different_dependency_filter_provider() {
        let other = Builder::new()
            .dependency_filters(|_dep| Filters::new())
            .build();
        assert_ne!(other, Preset::default());
    }

    #[test]
    fn test_default_preset() {
        let sut = Builder::new().cache(".some-cache").build();

        assert_eq!("default", sut.name());
        assert_eq!(".some-cache", sut.cache());
        assert_eq!("vendor", sut.vendor());
        assert_eq!(".vendor.yml", sut.spec());
        assert_eq!(".vendor-lock.yml", sut.spec_lock());
        assert!(!sut.force_filters());

        assert_eq!(Filters::new(), sut.global_filters());
        assert_eq!(".vendor-lock.yml", sut.spec_lock());

        let dep = &Dependency::new("some-url", "some-branch");
        assert_eq!(Filters::new(), sut.dependency_filters(dep));
    }
}
