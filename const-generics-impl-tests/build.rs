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
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let source_dir = "../tests";

    println!("cargo:rerun-if-changed={}", source_dir);

    copy_files(&source_dir, &out_dir).expect("could not copy required files");
}
