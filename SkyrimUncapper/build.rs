//!
//! @file build.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Builds/links the ASM and resource file for the uncapper.
//! @bug No known bugs.
//!

const RC_AUTHOR: &str = "Kasplat";
const RC_NAME: &str = "Skyrim Uncapper AE";
const RC_VERSION: &str = "2.0.1.0";
const RC_FILE: &str = "SkyrimUncapper.dll";

const HOOKS_FILE: &str = "src/hook_wrappers.S";

fn main() {
    // Embed resource information.
    let mut res = winres::WindowsResource::new();
    res.set("CompanyName", RC_AUTHOR);
    res.set("FileDescription", RC_NAME);
    res.set("FileVersion", RC_VERSION);
    res.set("InternalName", RC_FILE);
    res.set("LegalCopyright", "Copyright (C) 2023");
    res.set("OriginalFilename", RC_FILE);
    res.set("ProductName", RC_NAME);
    res.set("ProductVersion", RC_VERSION);
    res.compile().unwrap();

    // Compile our assembly hooks.
    println!("cargo:rerun-if-changed={}", HOOKS_FILE);
    vsprofile::VsProfile::get().asm_builder().file(HOOKS_FILE).compile("hooks");
}
