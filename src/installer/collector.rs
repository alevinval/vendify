use std::fs;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;
use walkdir::DirEntry;

use super::selector::Selector;

pub struct Collector {
    src_root: PathBuf,
    dst_root: PathBuf,
    selector: Selector,
}

pub struct CollectedPath {
    pub src: PathBuf,
    pub src_rel: PathBuf,
    pub dst: PathBuf,
}

impl Collector {
    pub fn new<P: AsRef<Path>>(src_root: &P, dst_root: &P, selector: Selector) -> Self {
        Collector {
            src_root: src_root.as_ref().to_owned(),
            dst_root: dst_root.as_ref().to_owned(),
            selector,
        }
    }

    pub fn iter(&self) -> Box<dyn Iterator<Item = CollectedPath> + '_> {
        Box::new(
            walkdir::WalkDir::new(self.src_root.as_path())
                .into_iter()
                .filter_entry(|entry| self.select(entry))
                .filter_map(Result::ok)
                .filter(|entry| entry.path().is_file())
                .map(|entry| self.to_collected_path(&entry)),
        )
    }

    fn to_collected_path(&self, entry: &DirEntry) -> CollectedPath {
        let src_rel = self.rel(entry);
        let dst = self.dst_root.join(&src_rel);
        CollectedPath {
            src: entry.path().to_owned(),
            src_rel,
            dst,
        }
    }

    fn select(&self, entry: &DirEntry) -> bool {
        let rel = self.rel(entry);
        if entry.path().is_dir() {
            self.selector.select_dir(&rel)
        } else {
            self.selector.select_path(&rel)
        }
    }

    fn rel(&self, entry: &DirEntry) -> PathBuf {
        entry
            .path()
            .strip_prefix(&self.src_root)
            .unwrap()
            .to_owned()
    }
}

impl CollectedPath {
    pub fn copy(&self) -> Result<()> {
        if let Some(parent) = self.dst.parent() {
            fs::create_dir_all(parent)?;
        };
        fs::copy(&self.src, &self.dst)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::read_as_str;
    use crate::test_utils::tempdir;
    use crate::test_utils::write_to;

    #[test]
    fn test_collected_path_copy() -> Result<()> {
        let from = tempdir().path().join("src/path/file.txt");
        fs::create_dir_all(from.parent().unwrap())?;
        write_to(&from, "some-file");

        let to = tempdir().path().join("dst/parent/file.txt");

        assert!(!to.exists());
        let sut = CollectedPath {
            src: from.clone(),
            src_rel: from,
            dst: to.clone(),
        };
        sut.copy()?;
        assert!(to.exists());

        let contents = read_as_str(to);
        assert_eq!("some-file", contents);

        Ok(())
    }
}
