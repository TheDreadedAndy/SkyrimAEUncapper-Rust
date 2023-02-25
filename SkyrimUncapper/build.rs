//!
//! @file build.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Builds/links the ASM and resource file for the uncapper.
//! @bug No known bugs.
//!

const NATIVE_WRAPPERS: &str = "src/skyrim/native_wrappers.cpp";

const RC_AUTHOR: &str = "Kasplat";
const RC_NAME: &str = "Skyrim Uncapper AE";
const RC_VERSION: &str = env!("CARGO_PKG_VERSION");
const RC_FILE: &str = "SkyrimUncapper.dll";

fn main() {
    // Always rerun this build script.
    println!("cargo:rerun-if-changed=../");

    // Build C++ exception nets.
    println!("cargo:rerun-if-changed={}", NATIVE_WRAPPERS);
    cc::Build::new().cpp(true).file(NATIVE_WRAPPERS).compile("nets");

    // Embed resource information.
    let mut res = winres::WindowsResource::new();
    let resource_file = format!("{}/uncapper.rc", std::env::var("OUT_DIR").unwrap());
    res.set("CompanyName", RC_AUTHOR);
    res.set("FileDescription", RC_NAME);
    res.set("FileVersion", RC_VERSION);
    res.set("InternalName", RC_FILE);
    res.set("LegalCopyright", "Copyright (C) 2023");
    res.set("OriginalFilename", RC_FILE);
    res.set("ProductName", RC_NAME);
    res.set("ProductVersion", RC_VERSION);
    res.write_resource_file(&resource_file).unwrap();

    // Win-res can't cross compile, but embed-resource can. Thus, we use winres to generate
    // the rc file nad embed-resource to embed it. It do be like that sometimes.
    embed_resource::compile(&resource_file);

    // Generate git version information.
    let std::process::Output { stdout, .. } = std::process::Command::new("git").args(&[
        "describe",
        "--always",
        "--dirty",
        "--tags"
    ]).output().unwrap();
    let version = String::from_utf8(stdout).unwrap();
    println!("cargo:rustc-env=UNCAPPER_GIT_VERSION={}", version.trim());
}
