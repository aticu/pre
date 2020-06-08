//! A build script to make sure the correct tests are run.
//!
//! This copies all non-blacklisted files from the shared `tests` directory to the output directory.
//!
//! This allows tests to pick them up.

use std::{
    env,
    fs::{copy, create_dir_all, read_dir},
    io,
    path::Path,
};

/// The variable, which enables writing of the tests in the source directory.
///
/// This is explicitly opt-in, because the cargo documentation states
///
/// > In general, build scripts should not modify any files outside of OUT_DIR.
/// > It may seem fine on the first blush, but it does cause problems when you use such crate as a
/// > dependency, because there's an implicit invariant that sources in `.cargo/registry` should be
/// > immutable. cargo won't allow such scripts when packaging.
///
/// While this test package is not intended to be ever released, it is better to be safe.
///
/// However it should be noted, that trybuild with it's `TRYBUILD=overwrite` setting behaves in a
/// similar fashion, so it should be ok.
const MUTABLE_SOURCE_VARIABLE: &'static str = "GENERATE_TESTS";

/// Copies all files from `from` to `to`, except if they are blacklisted.
fn copy_files(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();

    if from.is_dir() {
        create_dir_all(to)?;
        for entry in read_dir(from)? {
            let entry = entry?.path();
            copy_files(&entry, &to.join(&entry.components().last().unwrap()))?;
        }
    } else {
        println!("cargo:rerun-if-changed={}", from.display());
        copy(from, to)?;
    }

    Ok(())
}

fn main() {
    let out_dir = "generated_tests";
    let source_dir = "../tests";

    println!("cargo:rerun-if-changed={}", source_dir);
    println!("cargo:rerun-if-env-changed={}", MUTABLE_SOURCE_VARIABLE);

    if let Ok(mut val) = env::var(MUTABLE_SOURCE_VARIABLE) {
        val.make_ascii_lowercase();
        if val == "yes" || val == "true" || val == "1" {
            copy_files(&source_dir, &out_dir).expect("could not copy required files");
        }
    }
}
