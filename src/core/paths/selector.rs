use std::ffi::OsStr;
use std::path::Path;

use crate::core::Dependency;
use crate::core::Filters;
use crate::core::Spec;

pub struct PathSelector {
    filters: Filters,
}

impl PathSelector {
    pub fn new(spec: &Spec, dependency: &Dependency) -> Self {
        PathSelector {
            filters: spec.filters.clone().merge(&dependency.filters).to_owned(),
        }
    }

    pub fn select<P: AsRef<Path>>(&self, path: P) -> bool {
        let path = path.as_ref();
        !self.is_ignored(path) && self.is_target(path) && self.is_extension(path)
    }

    fn is_target(&self, path: &Path) -> bool {
        self.filters.targets.iter().any(path_matcher(path)) || self.filters.targets.is_empty()
    }

    fn is_ignored(&self, path: &Path) -> bool {
        self.filters.ignores.iter().any(path_matcher(path))
    }

    fn is_extension(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            self.filters.extensions.iter().any(extension_matcher(ext))
                || self.is_perfect_match(path)
        } else {
            self.is_perfect_match(path)
        }
    }

    fn is_perfect_match(&self, path: &Path) -> bool {
        self.filters.targets.iter().any(|t| path.eq(Path::new(t)))
    }
}

type MatcherFn<'a> = Box<dyn Fn(&String) -> bool + 'a>;

fn path_matcher(path: &Path) -> MatcherFn {
    Box::new(|base| path.starts_with(base))
}

fn extension_matcher(input: &OsStr) -> MatcherFn {
    Box::new(|ext| input.eq_ignore_ascii_case(ext))
}

#[cfg(test)]
mod tests {
    use super::PathSelector;
    use crate::core::Dependency;
    use crate::core::Filters;
    use crate::core::Spec;

    #[macro_export]
    macro_rules! svec {
        ($($elem:expr),+ $(,)?) => {{
            let v = vec![
                $( String::from($elem), )*
            ];
            v
        }};
    }

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
            .add_targets(&svec!["a"])
            .add_ignores(&svec!["b"])
            .add_extensions(&svec!["c"]);

        let dep = &mut Dependency::new("some-url", "some-branch");
        dep.filters
            .add_targets(&svec!["1"])
            .add_ignores(&svec!["2"])
            .add_extensions(&svec!["3"]);

        let sut = &PathSelector::new(spec, dep);

        assert_eq!(
            spec.filters.clone().merge(&dep.filters).to_owned(),
            sut.filters,
        );
    }

    #[test]
    fn test_selector_with_targets() {
        let filters = &mut Filters::new();
        filters
            .add_targets(&svec!["target/a", "readme.md"])
            .add_ignores(&svec!["ignored/a", "target/a/ignored"])
            .add_extensions(&svec!["proto"]);

        let sut = PathSelector {
            filters: filters.to_owned(),
        };

        assert_selection!(sut.select("target/a/file.proto"));
        assert_selection!(sut.select("readme.md"));

        assert_no_selection!(sut.select("target/a/file.txt"));
        assert_no_selection!(sut.select("target/a/ignored/file.proto"));
        assert_no_selection!(sut.select("target/noextension"));
        assert_no_selection!(sut.select("ignored/a/file.proto"));
    }

    #[test]
    fn test_selector_without_targets() {
        let filters = &mut Filters::new();
        filters
            .add_ignores(&svec!["ignored/a", "target/a/ignored"])
            .add_extensions(&svec!["proto"]);

        let sut = PathSelector {
            filters: filters.to_owned(),
        };

        assert_selection!(sut.select("target/a/file.proto"));

        assert_no_selection!(sut.select("readme.md"));
        assert_no_selection!(sut.select("target/a/file.txt"));
        assert_no_selection!(sut.select("target/a/ignored/file.proto"));
        assert_no_selection!(sut.select("target/noextension"));
        assert_no_selection!(sut.select("ignored/a/file.proto"));
    }
}
