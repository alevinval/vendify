use std::path::Path;
use std::path::PathBuf;

pub trait PathIterator {
    fn iter(&self) -> Box<dyn Iterator<Item = PathBuf>>;
}

pub struct WalkdirPathIterator {
    root: PathBuf,
}

/// Collects file paths with the matching extension in the target directory
impl WalkdirPathIterator {
    pub fn new<P: AsRef<Path>>(root: &P) -> Self {
        WalkdirPathIterator {
            root: root.as_ref().to_owned(),
        }
    }
}

impl PathIterator for WalkdirPathIterator {
    fn iter(&self) -> Box<dyn Iterator<Item = PathBuf>> {
        Box::new(
            walkdir::WalkDir::new(self.root.as_path())
                .into_iter()
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.into_path())
                .filter(|path| path.is_file()),
        )
    }
}
