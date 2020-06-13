use rustc_version::{version_meta, Channel};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    match version_meta() {
        Ok(version) if version.channel == Channel::Nightly => {
            println!("cargo:rustc-cfg=nightly");
        }
        _ => (),
    }
}
