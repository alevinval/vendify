use std::fs;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;
use walkdir::DirEntry;

use super::selector::Selector;

/// Returns an iterator of [`CollectedPath`].
pub struct Collector {
    selector: Selector,
}

/// Represents a file that has been collected, it allows to copy the file
/// from the source to the destination path.
///
/// Collected paths are aware of the relative path in respect to the
/// source folder.
pub struct CollectedPath {
    pub src: PathBuf,
    pub src_rel: PathBuf,
}

impl Collector {
    pub fn from(selector: Selector) -> Self {
        Self { selector }
    }

    pub fn collect<P: AsRef<Path>>(&self, from: &P) -> impl Iterator<Item = CollectedPath> + '_ {
        let from = from.as_ref().to_owned();
        let from_copy = from.clone();
        walkdir::WalkDir::new(&from)
            .into_iter()
            .filter_entry(move |entry| self.select_entry(&from, entry))
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_file())
            .map(move |entry| CollectedPath::new(&from_copy, &entry))
    }

    fn select_entry<P: AsRef<Path>>(&self, from: &P, entry: &DirEntry) -> bool {
        let rel = relative(from, entry);
        if entry.path().is_dir() {
            self.selector.select_dir(&rel)
        } else {
            self.selector.select_file(&rel)
        }
    }
}

impl From<Selector> for Collector {
    fn from(selector: Selector) -> Self {
        Collector::from(selector)
    }
}

impl CollectedPath {
    pub fn new<P: AsRef<Path>>(from: &P, entry: &DirEntry) -> CollectedPath {
        Self {
            src: entry.path().to_owned(),
            src_rel: relative(from, entry),
        }
    }

    /// Copies the collected file and its contents from the source to the
    /// destination path
    pub fn copy<P: AsRef<Path>>(&self, to: &P) -> Result<()> {
        let to = to.as_ref().join(&self.src_rel);
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent)?;
        };
        fs::copy(&self.src, &to)?;
        Ok(())
    }
}

fn relative<P: AsRef<Path>>(from: &P, entry: &DirEntry) -> PathBuf {
    entry
        .path()
        .strip_prefix(from)
        .unwrap_or_else(|_| entry.path())
        .to_path_buf()
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_utils::read_to_string;
    use crate::test_utils::tempdir;
    use crate::test_utils::write_to;

    #[test]
    fn test_collected_path_copy() -> Result<()> {
        let from = tempdir().path().join("src/path/file.txt");
        fs::create_dir_all(from.parent().unwrap())?;
        write_to(&from, "some-data");

        let to_parent_dir = tempdir().path().join("dst");
        let expected_to = to_parent_dir.join("path/file.txt");

        assert!(!to_parent_dir.exists());
        let sut = CollectedPath {
            src: from,
            src_rel: "path/file.txt".into(),
        };
        sut.copy(&to_parent_dir)?;
        assert!(expected_to.exists());

        let contents = read_to_string(&expected_to);
        assert_eq!("some-data", contents);

        Ok(())
    }
}
