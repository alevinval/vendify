use std::path::Path;
use std::path::PathBuf;

pub struct WalkdirPathIterator {
    root: PathBuf,
}

impl WalkdirPathIterator {
    pub fn new<P: AsRef<Path>>(root: &P) -> Self {
        WalkdirPathIterator {
            root: root.as_ref().to_owned(),
        }
    }

    pub fn iter(&self) -> Box<dyn Iterator<Item = PathBuf>> {
        Box::new(
            walkdir::WalkDir::new(self.root.as_path())
                .into_iter()
                .filter_map(|entry| entry.ok())
                .filter(|e| e.path().is_file())
                .map(|e| e.into_path()),
        )
    }
}
