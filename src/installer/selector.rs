use std::path::Path;

use crate::deps::Dependency;
use crate::filters::Filters;
use crate::spec::Spec;

pub struct Selector {
    filters: Filters,
}

impl Selector {
    pub fn new(spec: &Spec, dependency: &Dependency) -> Self {
        Self {
            filters: spec.filters.clone().merge(&dependency.filters).clone(),
        }
    }

    pub fn select_path<P: AsRef<Path>>(&self, path: P) -> bool {
        let path = path.as_ref();
        !self.is_ignored(path) && self.is_target(path) && self.is_extension(path)
    }

    pub fn select_dir<P: AsRef<Path>>(&self, dir: P) -> bool {
        let dir = dir.as_ref();
        if dir.as_os_str().is_empty() {
            return true;
        }
        !self.is_ignored(dir)
            && (self.filters.targets.is_empty()
                || Self::inverse_has_prefix(
                    &self.filters.targets,
                    &dir.to_path_buf().into_os_string().into_string().unwrap(),
                ))
    }

    fn is_target(&self, path: &Path) -> bool {
        self.filters
            .targets
            .iter()
            .any(|target| Self::starts_with(path, target))
            || self.filters.targets.is_empty()
    }

    fn is_ignored(&self, path: &Path) -> bool {
        self.filters
            .ignores
            .iter()
            .any(|ignore| Self::starts_with(path, ignore))
    }

    fn is_extension(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            self.filters
                .extensions
                .iter()
                .any(|target| ext.eq_ignore_ascii_case(target))
                || self.is_perfect_match(path)
        } else {
            self.is_perfect_match(path)
        }
    }

    fn is_perfect_match(&self, path: &Path) -> bool {
        self.filters
            .targets
            .iter()
            .map(Path::new)
            .any(|target| path == target)
    }

    fn inverse_has_prefix(paths: &[String], prefix: &String) -> bool {
        paths.iter().any(|path| {
            if prefix.len() > path.len() {
                prefix.starts_with(path)
            } else {
                path.starts_with(prefix)
            }
        })
    }

    fn starts_with(path: &Path, prefix: &str) -> bool {
        path.starts_with(prefix)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::filters::FilterKind;
    use crate::svec;

    macro_rules! assert_selection {
        ($cond:expr) => {{
            assert_eq!(true, $cond, "should have selected path")
        }};
    }

    macro_rules! assert_no_selection {
        ($cond:expr) => {{
            assert_eq!(false, $cond, "should not have selected path")
        }};
    }

    #[test]
    fn test_selector_combines_filters() {
        let spec = &mut Spec::new();
        spec.filters
            .add(FilterKind::Target(svec!["a"]))
            .add(FilterKind::Ignore(svec!["b"]))
            .add(FilterKind::Extension(svec!["c"]));

        let dep = &mut Dependency::new("some-url", "some-branch");
        dep.filters
            .add(FilterKind::Target(svec!["1"]))
            .add(FilterKind::Ignore(svec!["2"]))
            .add(FilterKind::Extension(svec!["3"]));

        let sut = &Selector::new(spec, dep);

        assert_eq!(
            spec.filters.clone().merge(&dep.filters).clone(),
            sut.filters,
        );
    }

    #[test]
    fn test_selector_with_targets() {
        let filters = &mut Filters::new();
        filters
            .add(FilterKind::Target(svec!["target/a", "readme.md"]))
            .add(FilterKind::Ignore(svec!["ignored/a", "target/a/ignored"]))
            .add(FilterKind::Extension(svec!["proto"]));

        let sut = Selector {
            filters: filters.clone(),
        };

        assert_selection!(sut.select_path("target/a/file.proto"));
        assert_selection!(sut.select_path("readme.md"));

        assert_no_selection!(sut.select_path("target/a/file.txt"));
        assert_no_selection!(sut.select_path("target/a/ignored/file.proto"));
        assert_no_selection!(sut.select_path("target/noextension"));
        assert_no_selection!(sut.select_path("ignored/a/file.proto"));
    }

    #[test]
    fn test_selector_without_targets() {
        let filters = &mut Filters::new();
        filters
            .add(FilterKind::Ignore(svec!["ignored/a", "target/a/ignored"]))
            .add(FilterKind::Extension(svec!["proto"]));

        let sut = Selector {
            filters: filters.clone(),
        };

        assert_selection!(sut.select_path("target/a/file.proto"));

        assert_no_selection!(sut.select_path("readme.md"));
        assert_no_selection!(sut.select_path("target/a/file.txt"));
        assert_no_selection!(sut.select_path("target/a/ignored/file.proto"));
        assert_no_selection!(sut.select_path("target/noextension"));
        assert_no_selection!(sut.select_path("ignored/a/file.proto"));
    }
}
