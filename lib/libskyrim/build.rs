//!
//! @file build.rs
//! @author Andrew Spaluding (Kasplat)
//! @brief Sets up the environment variable with the git revision.
//! @bug No known bugs.
//!

fn main() {
    // Generate git version information.
    let std::process::Output { stdout, .. } = std::process::Command::new("git").args(&[
        "describe",
        "--always",
        "--dirty",
        "--tags"
    ]).output().unwrap();
    let version = String::from_utf8(stdout).unwrap();
    println!("cargo:rustc-env=LIBSKYRIM_PLUGIN_VC_VERSION={}", version.trim());
}
