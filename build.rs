//! Makes the full release version (Cargo's version plus a package revision, if Cargo.toml
//! has one) available as APP_VERSION, so the About window shows the same version as android-webcam
//! releases.

use std::{env, fs};

fn main() {
    println!("cargo:rerun-if-changed=Cargo.toml");
    let version = env::var("CARGO_PKG_VERSION").unwrap();
    let manifest = fs::read_to_string("Cargo.toml").unwrap();
    let revision = manifest
        .lines()
        .find_map(|l| l.trim().strip_prefix("revision"))
        .and_then(|rest| rest.split('"').nth(1));
    let full = match revision {
        Some(rev) if !rev.is_empty() => format!("{}-{}", version, rev),
        _ => version,
    };
    println!("cargo:rustc-env=APP_VERSION={}", full);
}
