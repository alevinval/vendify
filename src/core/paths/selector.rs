use std::ffi::OsStr;
use std::path::Path;

use log::debug;

use crate::core::Dependency;
use crate::core::Spec;

pub struct PathSelector<'a> {
    vendor_spec: &'a Spec,
    dependency: &'a Dependency,
}

impl<'a> PathSelector<'a> {
    pub fn new(vendor_spec: &'a Spec, dependency: &'a Dependency) -> Self {
        PathSelector {
            vendor_spec,
            dependency,
        }
    }

    pub fn select<P: AsRef<Path>>(&self, path: P) -> bool {
        let path = path.as_ref();
        if self.is_ignored(path) {
            debug!("\t- {} [IGNORED]", path.display());
            return false;
        }
        if !self.is_target(path) {
            debug!("\t- {} [NOT TARGET]", path.display());
            return false;
        }
        if !self.is_extension(path) {
            debug!("\t- {} [IGNORED EXTENSION]", path.display());
            return false;
        }
        true
    }

    fn is_ignored(&self, path: &Path) -> bool {
        chained_any(
            &self.vendor_spec.ignores,
            &self.dependency.ignores,
            path_matcher(path),
        )
    }

    fn is_target(&self, path: &Path) -> bool {
        chained_any(
            &self.vendor_spec.targets,
            &self.dependency.targets,
            path_matcher(path),
        )
    }

    fn is_extension(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            chained_any(
                &self.vendor_spec.extensions,
                &self.dependency.extensions,
                extension_matcher(ext),
            )
        } else {
            false
        }
    }
}

type MatcherFn<'a> = Box<dyn Fn(&String) -> bool + 'a>;

fn path_matcher(path: &Path) -> MatcherFn {
    Box::new(|base| path.starts_with(base))
}

fn extension_matcher(input: &OsStr) -> MatcherFn {
    Box::new(|ext| input.eq_ignore_ascii_case(ext))
}

fn chained_any(a: &[String], b: &[String], f: MatcherFn) -> bool {
    a.iter().chain(b.iter()).any(f)
}

#[cfg(test)]
mod tests {
    use super::PathSelector;
    use crate::core::Dependency;
    use crate::core::Spec;

    macro_rules! svec {
        ($($elem:expr),+ $(,)?) => {{
            let v = vec![
                $( String::from($elem), )*
            ];
            v
        }};
    }

    #[test]
    fn test_selector_does_not_select_when_ignored() {
        let ignored_path_by_vendor = "/b";
        let ignored_file_by_vendor = "/a/b/c/file-1.txt";

        let ignored_path_by_dependency = "/c";
        let ignored_file_by_dependency = "/a/b/c/file-2.txt";

        let mut vendor_spec = Spec::new();
        vendor_spec.ignores = svec![ignored_path_by_vendor, ignored_file_by_vendor];

        let mut dependency = Dependency::new("some-url", "some-refname");
        dependency.ignores = svec![ignored_path_by_dependency, ignored_file_by_dependency];

        let sut = PathSelector::new(&vendor_spec, &dependency);

        assert!(
            !sut.select("/b/a/file-1.txt"),
            "should not be selected when path is ignored by vendor"
        );

        assert!(
            !sut.select("/c/b/a/file-1.txt"),
            "should not be selected when path is ignored by dependency"
        );

        assert!(
            !sut.select(ignored_file_by_vendor),
            "should not be selected when file is ignored by vendor"
        );

        assert!(
            !sut.select(ignored_file_by_dependency),
            "should not be selected when file is ignored by dependency"
        );
    }

    #[test]
    fn test_selector_selects_targets() {
        let mut vendor_spec = Spec::new();
        vendor_spec.targets = svec!["/vendor/path-1", "/vendor/path-2/file-1.txt"];
        vendor_spec.extensions = svec!["txt"];

        let mut dependency = Dependency::new("some-url", "some-refname");
        dependency.targets = svec!["/dep/path-1", "/dep/path-2/file-1.txt"];

        let sut = PathSelector::new(&vendor_spec, &dependency);

        assert!(
            sut.select("/vendor/path-1/file-1.txt"),
            "should select vendor path"
        );

        assert!(
            sut.select("/vendor/path-2/file-1.txt"),
            "should select vendor file path"
        );

        assert!(
            sut.select("/dep/path-1/file-1.txt"),
            "should select dependency path"
        );

        assert!(
            sut.select("/dep/path-2/file-1.txt"),
            "should select dependency file path"
        );
    }

    #[test]
    fn test_selector_does_not_select_ignored_extensions() {
        let mut vendor_spec = Spec::new();
        vendor_spec.targets = svec!["/a/path-1"];
        vendor_spec.extensions = svec!["txt"];

        let mut dependency = Dependency::new("some-url", "some-refname");
        dependency.extensions = svec!["proto"];

        let sut = PathSelector::new(&vendor_spec, &dependency);

        assert!(
            sut.select("/a/path-1/file-1.txt"),
            "should select vendor extension"
        );

        assert!(
            sut.select("/a/path-1/file-1.proto"),
            "should select dependency extension"
        );
        assert!(
            !sut.select("/a/path-1/file-1.md"),
            "should not select unknown extension"
        );
    }
}
