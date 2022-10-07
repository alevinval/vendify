use std::fs;
use std::path::Path;

use tempfile::NamedTempFile;
use tempfile::TempDir;

#[macro_export]
macro_rules! svec {
        ($($elem:expr),+ $(,)?) => {{
            let v = vec![
                $( String::from($elem), )*
            ];
            v
        }};
    }

pub fn tempdir() -> TempDir {
    match tempfile::tempdir() {
        Ok(dir) => dir,
        Err(_) => panic!("cannot create temporary folder"),
    }
}

pub fn tempfile() -> NamedTempFile {
    match tempfile::NamedTempFile::new() {
        Ok(file) => file,
        Err(_) => panic!("cannot create named temporary file"),
    }
}

pub fn write_to<P: AsRef<Path>>(dst: P, data: &str) {
    if let Err(err) = fs::write(&dst, data) {
        panic!("cannot write to {}: {}", dst.as_ref().display(), err)
    }
}

pub fn read_as_str<P: AsRef<Path>>(src: P) -> String {
    fs::read_to_string(src).expect("cannot read path")
}
