use std::path::Path;

use crate::filters::Filters;

/// Selects file or directory paths depending on whether the paths are allowed
/// based on the provided filters.
pub struct Selector {
    filters: Filters,
}

impl Selector {
    pub fn from(filters: Filters) -> Self {
        Self { filters }
    }

    /// Returns whether the path should be selected based on the filters.
    ///
    /// If the filepath is ignored, do not select
    /// If the filepath is a target, and has allowed extension, then select.
    pub fn select_file<P: AsRef<Path>>(&self, path: P) -> bool {
        let path = path.as_ref();
        !self.is_ignored(path) && self.is_target(path) && self.is_extension(path)
    }

    /// Returns whether the directory path should be selected based on the
    /// filters.
    ///
    /// All the target paths have to be compared against the current candidate
    /// directory path. Because we are running a recursive traversal from a root
    /// directory, filters might be targeting a path like `a/b`, but the
    /// traversal begins on directory `a`, then `a/b/`, and finally `a/b/c`
    ///
    /// When selecting directories, we must check if...
    ///
    ///  1) Any target path contains the current candidate as prefix
    ///     eg. 'a/' dir should be selected, because `a/b` is a target
    ///
    ///  2) If the current candidate contains as a prefix any of the targets
    ///     eg. `a/b/c` dir should be selected, because `a/b` is target
    pub fn select_dir<P: AsRef<Path>>(&self, dir: P) -> bool {
        let dir = dir.as_ref();

        // We want the Collector to traverse the root directory with respect
        // to the root path, this might result in an empty relative path, thus
        // always select it.
        if dir.as_os_str().is_empty() {
            return true;
        }

        !self.is_ignored(dir)
            && (self.filters.targets.is_empty()
                || Self::inverse_has_prefix(
                    &self.filters.targets,
                    &dir.to_path_buf()
                        .into_os_string()
                        .into_string()
                        .unwrap_or(String::new()),
                ))
    }

    /// Returns if the path is targeted.
    ///
    /// If there are no explicit targets, everything is a target.
    fn is_target(&self, path: &Path) -> bool {
        self.filters
            .targets
            .iter()
            .any(|target| path.starts_with(target))
            || self.filters.targets.is_empty()
    }

    /// Returns if the path is ignored.
    fn is_ignored(&self, path: &Path) -> bool {
        self.filters
            .ignores
            .iter()
            .any(|ignore| path.starts_with(ignore))
    }

    /// Returns if the path contains a targeted extension.
    ///
    /// If the path contains no extension, then we return true in case
    /// that the candidate path exactly matches any of the targets.
    fn is_extension(&self, path: &Path) -> bool {
        path.extension().map_or_else(
            || self.is_exact_target(path),
            |ext| {
                self.filters
                    .extensions
                    .iter()
                    .any(|target| ext.eq_ignore_ascii_case(target))
                    || self.is_exact_target(path)
            },
        )
    }

    fn is_exact_target(&self, path: &Path) -> bool {
        self.filters
            .targets
            .iter()
            .any(|target| path.to_string_lossy().eq_ignore_ascii_case(target))
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
    fn test_selector_with_targets() {
        let mut filters = Filters::new();
        filters
            .add(FilterKind::Target(svec!["target/a", "readme.md"]))
            .add(FilterKind::Ignore(svec!["ignored/a", "target/a/ignored"]))
            .add(FilterKind::Extension(svec!["proto"]));

        let sut = Selector { filters };

        assert_selection!(sut.select_file("target/a/file.proto"));
        assert_selection!(sut.select_file("readme.md"));

        assert_no_selection!(sut.select_file("target/a/file.txt"));
        assert_no_selection!(sut.select_file("target/a/ignored/file.proto"));
        assert_no_selection!(sut.select_file("target/noextension"));
        assert_no_selection!(sut.select_file("ignored/a/file.proto"));
    }

    #[test]
    fn test_selector_without_targets() {
        let mut filters = Filters::new();
        filters
            .add(FilterKind::Ignore(svec!["ignored/a", "target/a/ignored"]))
            .add(FilterKind::Extension(svec!["proto"]));

        let sut = Selector { filters };

        assert_selection!(sut.select_file("target/a/file.proto"));

        assert_no_selection!(sut.select_file("readme.md"));
        assert_no_selection!(sut.select_file("target/a/file.txt"));
        assert_no_selection!(sut.select_file("target/a/ignored/file.proto"));
        assert_no_selection!(sut.select_file("target/noextension"));
        assert_no_selection!(sut.select_file("ignored/a/file.proto"));
    }
}
