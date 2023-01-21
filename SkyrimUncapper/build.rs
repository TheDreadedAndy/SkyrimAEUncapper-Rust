//!
//! @file build.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Builds/links the ASM and resource file for the uncapper.
//! @bug No known bugs.
//!

use std::process::Command;
use std::path::PathBuf;

const RESOURCE_FILE: &str = "SkyrimUncapper.rc";
const RESOURCE_HEADER: &str = "resource.h";
const RC_BIN: &str = "C:\\Program Files (x86)\\Windows Kits\\10\\bin\\10.0.22000.0\\x64\\rc.exe";
const RC_INC: &str = "C:\\Program Files (x86)\\Windows Kits\\10\\Include\\10.0.22000.0\\um";
const RC_INC_SHARED: &str = "C:\\Program Files (x86)\\Windows Kits\\10\\Include\\10.0.22000.0\\shared";

fn main() {
    // Mark our RC as a dependency.
    println!("cargo:rerun-if-changed={}", RESOURCE_FILE);
    println!("cargo:rerun-if-changed={}", RESOURCE_HEADER);

    // Build the RC file.
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let output_file = out_dir.join("resource.res");
    Command::new(RC_BIN).args(&[
        "/D \"_UNICODE\"",
        "/D \"UNICODE\"",
        "/l 0x0409",
        "/nologo",
        "/I", RC_INC,
        "/I", RC_INC_SHARED,
        "/r",
        "/fo",
        output_file.as_os_str().to_str().unwrap(),
        RESOURCE_FILE
    ]).status().unwrap();

    // Create a static library out of the resource file.
    let ar_bin = PathBuf::from(std::env::var("LIBCLANG_PATH").unwrap()).join("llvm-ar.exe");
    let resource_lib = out_dir.join("resource.lib");
    Command::new(ar_bin).args(&[
        "rc",
        resource_lib.as_os_str().to_str().unwrap(),
        output_file.as_os_str().to_str().unwrap()
    ]).status().unwrap();

    // Link in the resource file.
    println!("cargo:rustc-link-search={}", out_dir.as_os_str().to_str().unwrap());
    println!("cargo:rustc-link-lib=static=resource");
}
