use std::path::Path;
use std::path::PathBuf;

pub struct PathCollector<'a> {
    root: &'a Path,
}

/// Collects file paths with the matching extension in the target directory
impl<'a> PathCollector<'a> {
    pub fn new<P: AsRef<Path>>(root: &'a P) -> Self {
        PathCollector {
            root: root.as_ref(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = PathBuf> + '_ {
        walkdir::WalkDir::new(&self.root)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.into_path())
            .filter(|path| path.is_file())
    }
}
