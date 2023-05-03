use std::fs;
use std::path::Path;
use std::sync::Arc;

use tempfile::TempDir;

use crate::deps::Dependency;
use crate::filters::FilterKind;
use crate::filters::Filters;
use crate::preset::Builder;
use crate::preset::Preset;

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
        Err(err) => panic!("cannot create temporary folder: {err}"),
    }
}

pub fn write_to<P: AsRef<Path>>(dst: P, data: &str) {
    if let Err(err) = fs::write(&dst, data) {
        panic!("cannot write to {}: {err}", dst.as_ref().display())
    }
}

pub fn read_to_string<P: AsRef<Path>>(src: &P) -> String {
    fs::read_to_string(src)
        .unwrap_or_else(|_| panic!("cannot read path {}", src.as_ref().display()))
}

pub fn build_preset() -> Arc<Preset> {
    preset_builder().build().into()
}

pub fn build_preset_with_fs(temp_dir: &TempDir) -> Arc<Preset> {
    let root = temp_dir.path();
    let tmp = |x: &str| root.join(x).into_os_string().into_string().unwrap();

    Arc::new(
        preset_builder()
            .name("test-preset")
            .cache(&tmp(".test-cache"))
            .vendor(&tmp(".test-vendor"))
            .spec(&tmp(".test-vendor.yml"))
            .spec_lock(&tmp(".test-vendor-lock.yml"))
            .build(),
    )
}

pub fn preset_builder() -> Builder {
    let mut filters = Filters::new();
    filters
        .add(FilterKind::Target(svec!("global/target/a")))
        .add(FilterKind::Ignore(svec!("global/ignore/a")))
        .add(FilterKind::Extension(svec!("txt")));

    let dependency_filters = |_dep: &Dependency| {
        let mut filters = Filters::new();
        filters
            .add(FilterKind::Target(svec!("dep/target/a")))
            .add(FilterKind::Ignore(svec!("dep/ignore/a")))
            .add(FilterKind::Extension(svec!("md")));
        filters
    };

    Builder::new()
        .name("test-preset")
        .cache(".test-cache")
        .vendor(".test-vendor")
        .spec(".test-vendor.yml")
        .spec_lock(".test-vendor-lock.yml")
        .global_filters(filters)
        .dependency_filters(dependency_filters)
}

pub struct TestContext {
    pub preset: Arc<Preset>,
    _temp_dir: TempDir,
}

impl TestContext {
    pub fn new() -> Self {
        let temp_dir = tempdir();
        TestContext {
            preset: build_preset_with_fs(&temp_dir),
            _temp_dir: temp_dir,
        }
    }
}

impl Default for TestContext {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&TestContext> for Arc<Preset> {
    fn from(ctx: &TestContext) -> Self {
        ctx.preset.clone()
    }
}
